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
  });

  const posts = await client.query(
    "SELECT id, title, content FROM posts ORDER BY created_at DESC",
  );

  console.log("Posts:", posts);
}

main().catch((err) => {
  console.error("Error:", err);
  process.exit(1);
});
