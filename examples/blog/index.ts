import { TinyCmsClient } from "tinycms/client";

const token = process.argv[2];

if (!token) {
  console.error("no token provided");
  console.error("run this script with the token as an argument");
  process.exit(1);
}

async function main() {
  const client = new TinyCmsClient({
    baseUrl: "http://localhost:3000",
    token,
  });

  const posts = await client.query(
    `SELECT
      p.id, p.title, p.slug,
      p.body, a.name as author_name
    FROM post p
    LEFT JOIN author a ON p.author::uuid = a.id
    ORDER BY p.created_at DESC`,
  );

  console.log("Posts:", posts);
}

main().catch((err) => {
  console.error("Error:", err);
  process.exit(1);
});
