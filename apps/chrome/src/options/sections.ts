export const SECTIONS = [
  {
    id: 'general',
    label: 'General',
  },
  {
    id: 'sites',
    label: 'Sites',
  },
] as const;

export type SectionId = (typeof SECTIONS)[number]['id'];
