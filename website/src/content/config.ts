import { defineCollection, z } from 'astro:content';

const docsCollection = defineCollection({
  type: 'content',
  schema: z.object({
    title: z.string(),
    // Ostala polja su opcionalna
  }),
});

export const collections = {
  'docs': docsCollection,
};

