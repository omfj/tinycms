export type ArrayItemType = "string" | "number" | "image" | "reference";

export interface FieldOption {
  label: string;
  value: string | number;
}

interface BaseField {
  name: string;
  required?: boolean;
  /** Display label shown in the admin UI (defaults to the field name) */
  title?: string;
  /** Helper text shown below the input */
  description?: string;
  /** Hide the field from the editor entirely */
  hidden?: boolean;
  /** Show the field but prevent editing */
  readOnly?: boolean;
}

export interface StringField extends BaseField {
  type: "string";
  placeholder?: string;
  /** Constrain to a fixed set of choices (renders a select) */
  options?: FieldOption[];
  /** Regex validation pattern */
  pattern?: string;
}

export interface TextField extends BaseField {
  type: "text";
  placeholder?: string;
  /** Number of visible rows */
  rows?: number;
}

export interface RichtextField extends BaseField {
  type: "richtext";
  placeholder?: string;
  /** Number of visible rows */
  rows?: number;
}

export interface NumberField extends BaseField {
  type: "number";
  placeholder?: string;
  /** Constrain to a fixed set of choices (renders a select) */
  options?: FieldOption[];
  min?: number;
  max?: number;
}

export interface BooleanField extends BaseField {
  type: "boolean";
}

export interface DateField extends BaseField {
  type: "date";
  /** Unix timestamp or date string used as lower bound */
  min?: number;
  /** Unix timestamp or date string used as upper bound */
  max?: number;
}

export interface UrlField extends BaseField {
  type: "url";
  placeholder?: string;
}

export interface SlugField extends BaseField {
  type: "slug";
  /** Field name to derive the slug from */
  source?: string;
}

export interface ImageField extends BaseField {
  type: "image";
  placeholder?: string;
  /** Accepted file types, e.g. "image/png,image/jpeg" */
  accept?: string;
}

export interface ReferenceField extends BaseField {
  type: "reference";
  /** Type names this field can point to */
  to?: string[];
}

export interface ArrayField extends BaseField {
  type: "array";
  /** Type of each array item */
  of?: ArrayItemType;
}

export type Field =
  | StringField
  | TextField
  | RichtextField
  | NumberField
  | BooleanField
  | DateField
  | UrlField
  | SlugField
  | ImageField
  | ReferenceField
  | ArrayField;

export interface TypeDef {
  name: string;
  fields: Field[];
}

export interface GitHubProvider {
  provider: "github";
  clientId: string;
  clientSecret: string;
}

export interface GoogleProvider {
  provider: "google";
  clientId: string;
  clientSecret: string;
}

export interface CredentialsProvider {
  provider: "credentials";
}

export type AuthProvider =
  | GitHubProvider
  | GoogleProvider
  | CredentialsProvider;

export interface AuthConfig {
  providers: AuthProvider[];
}

export interface StorageConfig {
  /** S3 bucket name */
  bucket: string;
  /** AWS region or 'auto' for Cloudflare R2 */
  region?: string;
  /** Access key ID — falls back to S3_ACCESS_KEY_ID env var */
  accessKeyId?: string;
  /** Secret access key — falls back to S3_SECRET_ACCESS_KEY env var */
  secretAccessKey?: string;
  /** Custom endpoint for S3-compatible services (R2, MinIO, Wasabi, …) */
  endpoint?: string;
}

export interface PostgresConfig {
  type: "postgres";
  url: string;
}

type DatabaseConfig = PostgresConfig;

export interface Config {
  database: DatabaseConfig;
  auth?: AuthConfig;
  storage?: StorageConfig;
  types: TypeDef[];
}

export function defineConfig(config: Config): Config {
  return config;
}

export function defineType(type: TypeDef): TypeDef {
  return type;
}

export function defineField<T extends Field>(field: T): T {
  return field;
}
