export const SECTIONS = [
  {
    id: 'general',
    label: 'General',
  },
] as const;

export type SectionId = (typeof SECTIONS)[number]['id'];
