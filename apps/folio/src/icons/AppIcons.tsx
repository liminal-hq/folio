// The app's icons: 16 px glyphs with a 1.3 px stroke and round caps, drawn inline
//
// (c) Copyright 2026 Liminal HQ, Scott Morris
// SPDX-License-Identifier: Apache-2.0 OR MIT

import type { ReactNode } from 'react';

function Glyph({ children }: { children: ReactNode }) {
	return (
		<svg
			width="16"
			height="16"
			viewBox="0 0 16 16"
			fill="none"
			stroke="currentColor"
			strokeWidth="1.3"
			strokeLinecap="round"
			strokeLinejoin="round"
			aria-hidden="true"
		>
			{children}
		</svg>
	);
}

export const NewBookIcon = () => (
	<Glyph>
		<path d="M3.5 1.5h6l3 3v10h-9z" />
		<path d="M8 7v4M6 9h4" />
	</Glyph>
);
export const OpenIcon = () => (
	<Glyph>
		<path d="M1.5 4.5v8.5h11l2-6h-11l-2 6" />
		<path d="M1.5 4.5h4l1.5 1.5h5v1" />
	</Glyph>
);
export const RecentIcon = () => (
	<Glyph>
		<circle cx="8" cy="8" r="6" />
		<path d="M8 4.5V8l2.5 1.5" />
	</Glyph>
);
export const ImportIcon = () => (
	<Glyph>
		<path d="M8 2v8M5 7l3 3 3-3" />
		<path d="M2.5 11v3h11v-3" />
	</Glyph>
);
export const SaveIcon = () => (
	<Glyph>
		<path d="M2.5 2.5h9l2 2v9h-11z" />
		<path d="M5 2.5v3.5h5V2.5M5 13.5V9.5h6v4" />
	</Glyph>
);
export const ExportIcon = () => (
	<Glyph>
		<path d="M8 10V2M5 5l3-3 3 3" />
		<path d="M2.5 9v5h11V9" />
	</Glyph>
);
export const PreferencesIcon = () => (
	<Glyph>
		<path d="M2 4.5h7M12 4.5h2M2 11.5h2M7 11.5h7" />
		<circle cx="10.5" cy="4.5" r="1.5" />
		<circle cx="5.5" cy="11.5" r="1.5" />
	</Glyph>
);
export const KeyboardIcon = () => (
	<Glyph>
		<rect x="1.5" y="4" width="13" height="8" rx="1.5" />
		<path d="M4 6.5h1M7 6.5h1M10 6.5h2M4 9.5h8" />
	</Glyph>
);
export const AboutIcon = () => (
	<Glyph>
		<circle cx="8" cy="8" r="6" />
		<path d="M8 7.5v3.5M8 5v.2" />
	</Glyph>
);
export const QuitIcon = () => (
	<Glyph>
		<path d="M6.5 2.5h-4v11h4M10 5l3 3-3 3M13 8H6" />
	</Glyph>
);
export const UndoIcon = () => (
	<Glyph>
		<path d="M5 3L2 6l3 3" />
		<path d="M2.5 6H10a3.5 3.5 0 010 7H6" />
	</Glyph>
);
export const RedoIcon = () => (
	<Glyph>
		<path d="M11 3l3 3-3 3" />
		<path d="M13.5 6H6a3.5 3.5 0 000 7h4" />
	</Glyph>
);
export const ZoomInIcon = () => (
	<Glyph>
		<circle cx="7" cy="7" r="4.5" />
		<path d="M10.5 10.5L14 14M7 5v4M5 7h4" />
	</Glyph>
);
export const ZoomOutIcon = () => (
	<Glyph>
		<circle cx="7" cy="7" r="4.5" />
		<path d="M10.5 10.5L14 14M5 7h4" />
	</Glyph>
);

export const MenuBarIcon = () => (
	<Glyph>
		<rect x="1.5" y="2.5" width="13" height="11" rx="1.5" />
		<path d="M1.5 5.5h13M4 4h1M7 4h1" />
	</Glyph>
);
export const RibbonIcon = () => (
	<Glyph>
		<rect x="1.5" y="2.5" width="13" height="4" rx="1" />
		<path d="M1.5 9.5h13M1.5 12.5h8" />
	</Glyph>
);
export const ExpertIcon = () => (
	<Glyph>
		<path d="M8 2l1.8 3.8 4.2.6-3 3 .7 4.2L8 11.5l-3.7 2.1.7-4.2-3-3 4.2-.6z" />
	</Glyph>
);
export const JaxIcon = () => (
	<Glyph>
		<circle cx="8" cy="8.5" r="5.5" />
		<circle cx="6" cy="7.5" r=".6" fill="currentColor" />
		<circle cx="10" cy="7.5" r=".6" fill="currentColor" />
		<path d="M6.5 10.5c.9.7 2.1.7 3 0" />
	</Glyph>
);
export const ViewIcon = () => (
	<Glyph>
		<path d="M1.5 8S4 3.5 8 3.5 14.5 8 14.5 8 12 12.5 8 12.5 1.5 8 1.5 8z" />
		<circle cx="8" cy="8" r="2" />
	</Glyph>
);
export const EditIcon = () => (
	<Glyph>
		<path d="M3 13l.5-3L11 2.5l2.5 2.5L6 12.5z" />
	</Glyph>
);

export const BookToolsIcon = () => (
	<Glyph>
		<path d="M9.5 2.5a3 3 0 00-2.8 4L2 11.2 4.8 14l4.7-4.7a3 3 0 004-2.8l-2 2-2-.5-.5-2z" />
	</Glyph>
);

export const CloseBookIcon = () => (
	<Glyph>
		<path d="M3.5 1.5h6l3 3v10h-9z" />
		<path d="M6 7l4 4M10 7l-4 4" />
	</Glyph>
);

/** The Folio mark for the title bar's menu button: an open spread with the slit of light. */
export function AppMarkIcon({ size = 18 }: { size?: number }) {
	return (
		<svg width={size} height={size} viewBox="0 0 64 64" aria-hidden="true">
			<defs>
				<linearGradient
					id="folio-mark-stroke"
					x1="12"
					y1="10"
					x2="52"
					y2="54"
					gradientUnits="userSpaceOnUse"
				>
					<stop offset="0" stopColor="#5d3147" />
					<stop offset="0.55" stopColor="#cf5d22" />
					<stop offset="1" stopColor="#f19b18" />
				</linearGradient>
			</defs>
			<rect x="2" y="2" width="60" height="60" rx="16" fill="#0e0d12" />
			<path
				d="M28.5 19C22 16.5 16 17 12.5 19V45.5C16 43.5 22 43 28.5 45.5"
				fill="none"
				stroke="url(#folio-mark-stroke)"
				strokeWidth="3.4"
				strokeLinecap="round"
				strokeLinejoin="round"
			/>
			<path
				d="M35.5 19C42 16.5 48 17 51.5 19V45.5C48 43.5 42 43 35.5 45.5"
				fill="none"
				stroke="url(#folio-mark-stroke)"
				strokeWidth="3.4"
				strokeLinecap="round"
				strokeLinejoin="round"
			/>
			<path
				d="M32 22V44"
				stroke="#f7e2ba"
				strokeOpacity="0.85"
				strokeWidth="2.6"
				strokeLinecap="round"
			/>
		</svg>
	);
}

export const DuplicateIcon = () => (
	<Glyph>
		<rect x="5.5" y="5.5" width="8" height="8" rx="1.2" />
		<path d="M10.5 3.5v-1a1 1 0 0 0-1-1h-7a1 1 0 0 0-1 1v7a1 1 0 0 0 1 1h1" />
	</Glyph>
);
export const BringToFrontIcon = () => (
	<Glyph>
		<rect x="1.5" y="1.5" width="8" height="8" rx="1.2" />
		<rect x="6.5" y="6.5" width="8" height="8" rx="1.2" fill="currentColor" fillOpacity="0.25" />
	</Glyph>
);
export const SendToBackIcon = () => (
	<Glyph>
		<rect x="1.5" y="1.5" width="8" height="8" rx="1.2" fill="currentColor" fillOpacity="0.25" />
		<rect x="6.5" y="6.5" width="8" height="8" rx="1.2" />
	</Glyph>
);
export const LockIcon = () => (
	<Glyph>
		<rect x="3" y="7" width="10" height="7" rx="1.4" />
		<path d="M5.5 7V5a2.5 2.5 0 0 1 5 0v2" />
	</Glyph>
);
export const UnlockIcon = () => (
	<Glyph>
		<rect x="3" y="7" width="10" height="7" rx="1.4" />
		<path d="M5.5 7V5a2.5 2.5 0 0 1 4.8-1" />
	</Glyph>
);
export const TrashIcon = () => (
	<Glyph>
		<path d="M2.5 4h11M6 4V2.5h4V4M4 4l.7 9.5h6.6L12 4" />
	</Glyph>
);

export const MoveLeftIcon = () => (
	<Glyph>
		<path d="M13 8H3M7 4L3 8l4 4" />
	</Glyph>
);
export const MoveRightIcon = () => (
	<Glyph>
		<path d="M3 8h10M9 4l4 4-4 4" />
	</Glyph>
);
export const AddPagesIcon = () => (
	<Glyph>
		<path d="M3.5 1.5h6l3 3v10h-9z" />
		<path d="M8 7v4M6 9h4" />
	</Glyph>
);
