// Saves and opens `.folio` files for real, and checks what the format promises
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::fs;
use std::io::{Read, Write};

use folio_model::Book;
use folio_package::{FileRole, Manifest, Package, PackageError, SaveOptions, MIMETYPE};
use serde_json::json;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

fn book() -> Book {
    serde_json::from_value(json!({
        "schema": 1,
        "id": "bk_test",
        "title": "The Otter Who Couldn't Sleep",
        "authors": [{ "name": "Mom", "role": "author" }],
        "language": "en-CA",
        "kind": "picture",
        "layout": "fixed",
        "trim": { "w": 612, "h": 612 },
        "bleed": 9,
        "margins": { "top": 36, "bottom": 36, "inside": 36, "outside": 36 },
        "grid": { "columns": 6, "gutter": 12, "baseline": null },
        "spreads": [],
        "metadata": { "isbn": { "paperback": null, "hardcover": null, "ebook": null }, "description": "" }
    }))
    .unwrap()
}

fn entries(path: &std::path::Path) -> Vec<String> {
    let mut archive = ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
    (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_owned())
        .collect()
}

#[test]
fn a_saved_book_opens_with_the_same_content() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("otter.folio");
    let mut package = Package::new(book());
    package
        .set_flow_text(
            "text/flow-main.md",
            "# The keeper's kettle {style=h1}\n\nMara had kept the light.",
        )
        .unwrap();
    package.settings_mut()["lastTarget"] = json!("kindle");
    package.save(&file, &SaveOptions::default()).unwrap();

    let opened = Package::open(&file).unwrap();
    assert_eq!(opened.book().title, "The Otter Who Couldn't Sleep");
    assert_eq!(
        opened.flow_text("text/flow-main.md"),
        Some("# The keeper's kettle {style=h1}\n\nMara had kept the light.")
    );
    assert_eq!(opened.settings()["lastTarget"], "kindle");
}

#[test]
fn the_mimetype_entry_is_first_stored_and_has_no_extra_field() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("otter.folio");
    Package::new(book())
        .save(&file, &SaveOptions::default())
        .unwrap();

    let bytes = fs::read(&file).unwrap();
    assert_eq!(&bytes[0..4], b"PK\x03\x04", "a zip local header");
    assert_eq!(
        u16::from_le_bytes([bytes[8], bytes[9]]),
        0,
        "stored, not deflated"
    );
    assert_eq!(
        u16::from_le_bytes([bytes[26], bytes[27]]),
        8,
        "the name is `mimetype`"
    );
    assert_eq!(
        u16::from_le_bytes([bytes[28], bytes[29]]),
        0,
        "no extra field"
    );
    assert_eq!(&bytes[30..38], b"mimetype");
    // So `file(1)` and file managers can read the type straight from the start of the file.
    assert_eq!(&bytes[38..38 + MIMETYPE.len()], MIMETYPE.as_bytes());
    assert_eq!(entries(&file)[0], "mimetype");
}

#[test]
fn the_manifest_lists_every_file_with_its_hash_and_role() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("otter.folio");
    let mut package = Package::new(book());
    let image = package.add_image(b"not really a jpeg".to_vec(), "JPG");
    package.save(&file, &SaveOptions::default()).unwrap();

    let mut archive = ZipArchive::new(fs::File::open(&file).unwrap()).unwrap();
    let mut text = String::new();
    archive
        .by_name("META-INF/manifest.json")
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    let manifest: Manifest = serde_json::from_str(&text).unwrap();
    assert_eq!(manifest.format, "folio");
    assert_eq!(manifest.format_version, "1.0");
    let listed = manifest
        .files
        .iter()
        .find(|f| f.path == image)
        .expect("the picture is listed");
    assert_eq!(listed.role, FileRole::Image);
    assert_eq!(listed.media_type, "image/jpeg");
    assert_eq!(
        listed.sha256.as_deref(),
        Some(
            image
                .trim_start_matches("assets/images/")
                .trim_end_matches(".jpg")
        )
    );
    assert!(manifest
        .files
        .iter()
        .any(|f| f.path == "book.json" && f.role == FileRole::Book));
}

#[test]
fn json_is_two_space_indented_with_a_trailing_newline() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("otter.folio");
    Package::new(book())
        .save(&file, &SaveOptions::default())
        .unwrap();
    let mut archive = ZipArchive::new(fs::File::open(&file).unwrap()).unwrap();
    let mut text = String::new();
    archive
        .by_name("book.json")
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    assert!(text.starts_with("{\n  \"schema\": 1,\n  \"id\": \"bk_test\""));
    assert!(text.ends_with("}\n"));
}

#[test]
fn the_same_picture_added_twice_is_stored_once() {
    let mut package = Package::new(book());
    let one = package.add_image(vec![1, 2, 3], "png");
    let two = package.add_image(vec![1, 2, 3], ".PNG");
    let other = package.add_image(vec![9, 9], "png");
    assert_eq!(one, two);
    assert_ne!(one, other);
    assert_eq!(package.asset_paths().count(), 2);
}

#[test]
fn pictures_survive_a_save_as_and_a_save_over_without_being_loaded() {
    let dir = tempfile::tempdir().unwrap();
    let first = dir.path().join("one.folio");
    let second = dir.path().join("two.folio");
    let mut package = Package::new(book());
    let image = package.add_image(vec![7; 4096], "png");
    package.save(&first, &SaveOptions::default()).unwrap();

    // Open, save as another file, then save over that one: the picture is copied each time.
    let mut opened = Package::open(&first).unwrap();
    opened.save(&second, &SaveOptions::default()).unwrap();
    opened
        .set_flow_text("text/flow-main.md", "more words")
        .unwrap();
    opened.save(&second, &SaveOptions::default()).unwrap();

    let reopened = Package::open(&second).unwrap();
    assert_eq!(reopened.asset_bytes(&image).unwrap(), vec![7; 4096]);
    assert_eq!(reopened.flow_text("text/flow-main.md"), Some("more words"));
    // The original is untouched.
    assert_eq!(
        Package::open(&first).unwrap().asset_bytes(&image).unwrap(),
        vec![7; 4096]
    );
}

#[test]
fn a_copy_is_written_without_the_package_becoming_that_file() {
    let dir = tempfile::tempdir().unwrap();
    let real = dir.path().join("real.folio");
    let copy = dir.path().join("recovery").join("copy.folio");
    let mut package = Package::new(book());
    let saved = package.add_image(vec![1; 2048], "png");
    package.save(&real, &SaveOptions::default()).unwrap();

    // A picture added since, and a change to the words: the copy has both, and the real file neither.
    let added = package.add_image(vec![2; 2048], "png");
    package
        .set_flow_text("text/flow-main.md", "newer words")
        .unwrap();
    fs::create_dir_all(copy.parent().unwrap()).unwrap();
    let options = SaveOptions {
        keep_backup: false,
        ..SaveOptions::default()
    };
    package.save_copy(&copy, &options).unwrap();
    package.save_copy(&copy, &options).unwrap();

    let from_copy = Package::open(&copy).unwrap();
    assert_eq!(from_copy.asset_bytes(&saved).unwrap(), vec![1; 2048]);
    assert_eq!(from_copy.asset_bytes(&added).unwrap(), vec![2; 2048]);
    assert_eq!(
        from_copy.flow_text("text/flow-main.md"),
        Some("newer words")
    );
    let from_real = Package::open(&real).unwrap();
    assert_eq!(from_real.asset_paths().count(), 1);
    assert_eq!(from_real.flow_text("text/flow-main.md"), None);

    // The package still reads from the real file, so removing the copy loses nothing, and a save
    // afterwards still writes the real file whole.
    fs::remove_file(&copy).unwrap();
    assert_eq!(package.asset_bytes(&saved).unwrap(), vec![1; 2048]);
    package.save(&real, &SaveOptions::default()).unwrap();
    let after = Package::open(&real).unwrap();
    assert_eq!(after.asset_bytes(&added).unwrap(), vec![2; 2048]);
    assert_eq!(after.flow_text("text/flow-main.md"), Some("newer words"));
    // No half-written file is left beside the copy.
    assert!(!dir.path().join("recovery").join("copy.folio.tmp").exists());
}

#[test]
fn a_save_keeps_the_previous_one_as_a_backup() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("otter.folio");
    let mut package = Package::new(book());
    package.set_flow_text("text/flow-main.md", "first").unwrap();
    package.save(&file, &SaveOptions::default()).unwrap();
    package
        .set_flow_text("text/flow-main.md", "second")
        .unwrap();
    package.save(&file, &SaveOptions::default()).unwrap();

    let backup = dir.path().join("otter.folio~");
    assert_eq!(
        Package::open(&file).unwrap().flow_text("text/flow-main.md"),
        Some("second")
    );
    assert_eq!(
        Package::open(&backup)
            .unwrap()
            .flow_text("text/flow-main.md"),
        Some("first")
    );
    assert!(
        !dir.path().join("otter.folio.tmp").exists(),
        "no temporary file is left"
    );
}

#[test]
fn the_backup_can_be_turned_off() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("otter.folio");
    let options = SaveOptions {
        keep_backup: false,
        ..SaveOptions::default()
    };
    let mut package = Package::new(book());
    package.save(&file, &options).unwrap();
    package.save(&file, &options).unwrap();
    assert!(!dir.path().join("otter.folio~").exists());
}

#[test]
fn a_failed_save_leaves_the_old_file_alone() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("otter.folio");
    let mut package = Package::new(book());
    package.save(&file, &SaveOptions::default()).unwrap();
    let before = fs::read(&file).unwrap();

    // A folder where the temporary file must go makes the write fail.
    fs::create_dir(dir.path().join("otter.folio.tmp")).unwrap();
    assert!(package.save(&file, &SaveOptions::default()).is_err());
    assert_eq!(fs::read(&file).unwrap(), before);
}

#[test]
fn unknown_files_and_unknown_keys_survive_a_save() {
    let dir = tempfile::tempdir().unwrap();
    let first = dir.path().join("one.folio");
    let second = dir.path().join("two.folio");

    // A package written by a "newer Folio": an extra file and an extra key in book.json.
    let mut package = Package::new(book());
    package.save(&first, &SaveOptions::default()).unwrap();
    let mut archive = ZipArchive::new(fs::File::open(&first).unwrap()).unwrap();
    let mut book_json = String::new();
    archive
        .by_name("book.json")
        .unwrap()
        .read_to_string(&mut book_json)
        .unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&book_json).unwrap();
    value["futureThing"] = json!({ "answer": 42 });
    let mut zip = ZipWriter::new(fs::File::create(dir.path().join("future.folio")).unwrap());
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        let name = entry.name().to_owned();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if name == "book.json" {
            bytes = serde_json::to_vec_pretty(&value).unwrap();
        }
        zip.start_file(name, stored).unwrap();
        zip.write_all(&bytes).unwrap();
    }
    zip.start_file("notes/idea.txt", stored).unwrap();
    zip.write_all(b"keep me").unwrap();
    zip.finish().unwrap();

    let mut opened = Package::open(dir.path().join("future.folio")).unwrap();
    opened.save(&second, &SaveOptions::default()).unwrap();

    let reopened = Package::open(&second).unwrap();
    assert_eq!(reopened.extra_file("notes/idea.txt"), Some(&b"keep me"[..]));
    assert_eq!(reopened.book().extensions["futureThing"]["answer"], 42);
}

#[test]
fn files_that_are_not_folio_books_are_refused_with_a_reason() {
    let dir = tempfile::tempdir().unwrap();

    let text = dir.path().join("plain.txt");
    fs::write(&text, "hello").unwrap();
    assert!(matches!(Package::open(&text), Err(PackageError::Zip(_))));

    let wrong = dir.path().join("other.zip");
    let mut zip = ZipWriter::new(fs::File::create(&wrong).unwrap());
    zip.start_file(
        "mimetype",
        SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
    )
    .unwrap();
    zip.write_all(b"application/epub+zip").unwrap();
    zip.finish().unwrap();
    let error = Package::open(&wrong).unwrap_err();
    assert!(matches!(error, PackageError::NotFolio(_)), "{error}");

    let missing = dir.path().join("nope.folio");
    assert!(matches!(
        Package::open(&missing),
        Err(PackageError::Io { .. })
    ));
}

#[test]
fn a_book_from_a_newer_schema_is_refused_not_misread() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("otter.folio");
    let mut package = Package::new(book());
    package.save(&file, &SaveOptions::default()).unwrap();

    let rewritten = dir.path().join("newer.folio");
    let mut archive = ZipArchive::new(fs::File::open(&file).unwrap()).unwrap();
    let mut zip = ZipWriter::new(fs::File::create(&rewritten).unwrap());
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        let name = entry.name().to_owned();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if name == "book.json" {
            let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            value["schema"] = json!(99);
            bytes = serde_json::to_vec(&value).unwrap();
        }
        zip.start_file(name, stored).unwrap();
        zip.write_all(&bytes).unwrap();
    }
    zip.finish().unwrap();

    assert!(matches!(
        Package::open(&rewritten),
        Err(PackageError::NewerSchema {
            found: 99,
            supported: 2
        })
    ));
}

#[test]
fn paths_that_could_escape_the_package_are_refused() {
    let mut package = Package::new(book());
    for bad in [
        "../evil.md",
        "text/../../x.md",
        "/text/a.md",
        "text//a.md",
        "text/a.txt",
        "book.md",
    ] {
        assert!(
            matches!(
                package.set_flow_text(bad, "x"),
                Err(PackageError::BadPath(_))
            ),
            "{bad} should be refused"
        );
    }
    assert!(package.set_flow_text("text/flow-main.md", "ok").is_ok());
}

#[test]
fn the_autosave_journal_is_not_carried_into_a_saved_book() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("with-journal.folio");
    let mut package = Package::new(book());
    package.save(&source, &SaveOptions::default()).unwrap();

    let mut archive = ZipArchive::new(fs::File::open(&source).unwrap()).unwrap();
    let journalled = dir.path().join("journalled.folio");
    let mut zip = ZipWriter::new(fs::File::create(&journalled).unwrap());
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        let name = entry.name().to_owned();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        zip.start_file(name, stored).unwrap();
        zip.write_all(&bytes).unwrap();
    }
    zip.start_file("recovery/journal-1.ndjson", stored).unwrap();
    zip.write_all(b"{}\n").unwrap();
    zip.finish().unwrap();

    let mut opened = Package::open(&journalled).unwrap();
    let resaved = dir.path().join("resaved.folio");
    opened.save(&resaved, &SaveOptions::default()).unwrap();
    assert!(!entries(&resaved)
        .iter()
        .any(|name| name.starts_with("recovery/")));
}

#[test]
fn a_font_the_book_keeps_is_stored_once_by_its_hash_and_listed_with_its_media_type() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("otter.folio");
    let mut package = Package::new(book());
    let one = package.add_font(vec![1, 2, 3], "TTF");
    let two = package.add_font(vec![1, 2, 3], ".ttf");
    let cff = package.add_font(vec![9, 9], "otf");
    let odd = package.add_font(vec![7], "woff2");
    assert_eq!(one, two);
    assert!(one.starts_with("assets/fonts/") && one.ends_with(".ttf"));
    assert!(cff.ends_with(".otf"));
    assert!(odd.ends_with(".ttf"), "an unknown extension is called ttf");

    package.book_mut().fonts = vec![folio_model::BookFont {
        family: "Zed Sans".into(),
        weight: 400,
        italic: false,
        path: one.clone(),
        index: 0,
    }];
    package.save(&file, &SaveOptions::default()).unwrap();

    let mut archive = ZipArchive::new(fs::File::open(&file).unwrap()).unwrap();
    let mut text = String::new();
    archive
        .by_name("META-INF/manifest.json")
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    let manifest: Manifest = serde_json::from_str(&text).unwrap();
    let listed = |path: &str| manifest.files.iter().find(|f| f.path == path).unwrap();
    assert_eq!(listed(&one).role, FileRole::Font);
    assert_eq!(listed(&one).media_type, "font/ttf");
    assert_eq!(listed(&cff).media_type, "font/otf");

    let mut reopened = Package::open(&file).unwrap();
    assert_eq!(reopened.book().fonts[0].family, "Zed Sans");
    assert_eq!(reopened.asset_bytes(&one).unwrap(), vec![1, 2, 3]);
    assert!(reopened.remove_asset(&one));
    assert!(!reopened.remove_asset(&one));
    assert!(reopened.asset_bytes(&one).is_err());
}

#[test]
fn the_styles_are_the_books_in_memory_but_their_own_file_in_the_package() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("otter.folio");
    let mut with_styles = book();
    with_styles.styles = serde_json::from_value(json!({
        "look": "classic",
        "fontFamily": "Literata",
        "swatches": [{ "id": "ink", "name": "Ink", "display": "#1c1b20" }],
        "styles": { "body": { "name": "Body", "sizePt": 10.5, "family": "Inter" } }
    }))
    .unwrap();
    Package::new(with_styles.clone())
        .save(&file, &SaveOptions::default())
        .unwrap();

    let mut archive = ZipArchive::new(fs::File::open(&file).unwrap()).unwrap();
    let mut read = |name: &str| {
        let mut text = String::new();
        archive
            .by_name(name)
            .unwrap()
            .read_to_string(&mut text)
            .unwrap();
        serde_json::from_str::<serde_json::Value>(&text).unwrap()
    };
    assert!(read("book.json").get("styles").is_none());
    let file_styles = read("styles.json");
    assert_eq!(file_styles["fontFamily"], "Literata");
    assert_eq!(file_styles["styles"]["body"]["family"], "Inter");
    assert_eq!(file_styles["swatches"][0]["name"], "Ink");

    assert_eq!(Package::open(&file).unwrap().book(), &with_styles);
}
