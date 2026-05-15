import { config as loadEnv } from "dotenv";
import { fileURLToPath } from "node:url";
import { defineConfig, defineField, defineType } from "tinycms/config";

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
    providers: [{ provider: "credentials" }],
  },

  storage: {
    bucket: "my-shop-assets",
    region: "eu-west-1",
    endpoint: process.env.S3_ENDPOINT,
    accessKeyId: process.env.S3_ACCESS_KEY_ID,
    secretAccessKey: process.env.S3_SECRET_ACCESS_KEY,
  },

  types: [
    defineType({
      name: "product",
      fields: [
        defineField({ name: "name", type: "string", required: true }),
        defineField({ name: "slug", type: "slug", source: "name" }),
        defineField({ name: "description", type: "richtext" }),
        defineField({ name: "price", type: "number", required: true }),
        defineField({ name: "compare_at_price", type: "number" }),
        defineField({ name: "sku", type: "string" }),
        defineField({ name: "in_stock", type: "boolean" }),
        defineField({ name: "featured_image", type: "image" }),
        defineField({ name: "category", type: "reference", to: ["category"] }),
        defineField({ name: "seo_title", type: "string" }),
        defineField({ name: "seo_description", type: "text" }),
      ],
    }),

    defineType({
      name: "category",
      fields: [
        defineField({ name: "name", type: "string", required: true }),
        defineField({ name: "slug", type: "slug", source: "name" }),
        defineField({ name: "description", type: "text" }),
        defineField({ name: "image", type: "image" }),
        defineField({ name: "parent", type: "reference", to: ["category"] }),
      ],
    }),

    defineType({
      name: "collection",
      fields: [
        defineField({ name: "title", type: "string", required: true }),
        defineField({ name: "slug", type: "slug", source: "title" }),
        defineField({ name: "description", type: "text" }),
        defineField({ name: "image", type: "image" }),
      ],
    }),
  ],
});
