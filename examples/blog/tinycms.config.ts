import { config as loadEnv } from "dotenv";
import { fileURLToPath } from "node:url";
import { defineConfig, defineField, defineType } from "tinycms";

loadEnv({
  path: fileURLToPath(new URL(".env", import.meta.url)),
  quiet: true,
  override: true,
});

export default defineConfig({
  database: {
    type: "postgres",
    url: process.env.DATABASE_URL!,
  },

  auth: {
    providers: [
      {
        provider: "github",
        clientId: process.env.GITHUB_CLIENT_ID!,
        clientSecret: process.env.GITHUB_CLIENT_SECRET!,
      },
    ],
  },

  storage: {
    bucket: "my-blog-assets",
    region: "auto",
    endpoint: process.env.S3_ENDPOINT,
    accessKeyId: process.env.S3_ACCESS_KEY_ID,
    secretAccessKey: process.env.S3_SECRET_ACCESS_KEY,
  },

  types: [
    defineType({
      name: "post",
      fields: [
        defineField({ name: "title", type: "string", required: true }),
        defineField({ name: "slug", type: "slug", source: "title" }),
        defineField({ name: "excerpt", type: "text" }),
        defineField({ name: "body", type: "richtext", required: true }),
        defineField({ name: "cover_image", type: "image" }),
        defineField({ name: "author", type: "reference", to: ["author"] }),
        defineField({ name: "category", type: "reference", to: ["category"] }),
        defineField({ name: "published_at", type: "date" }),
        defineField({ name: "seo_title", type: "string" }),
        defineField({ name: "seo_description", type: "text" }),
      ],
    }),

    defineType({
      name: "author",
      fields: [
        defineField({ name: "name", type: "string", required: true }),
        defineField({ name: "bio", type: "text" }),
        defineField({ name: "avatar", type: "image" }),
        defineField({ name: "email", type: "string" }),
        defineField({ name: "url", type: "url" }),
      ],
    }),

    defineType({
      name: "category",
      fields: [
        defineField({ name: "name", type: "string", required: true }),
        defineField({ name: "slug", type: "slug", source: "name" }),
        defineField({ name: "description", type: "text" }),
      ],
    }),
  ],
});
