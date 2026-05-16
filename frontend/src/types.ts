export type User = {
  id: string;
  email: string;
  name: string | null;
  role: "admin" | "editor" | "viewer" | string;
  status: "active" | "pending" | "suspended" | string;
};

export type WorkspaceSettings = {
  id: string;
  name: string;
  require_approval: boolean;
  default_role: string;
  created_at: string;
  updated_at: string;
};

export type ArrayItemType = "string" | "number" | "image" | "reference";

export type FieldOption = {
  label: string;
  value: string | number;
};

type BaseField = {
  name: string;
  required?: boolean;
  title?: string;
  description?: string;
  hidden?: boolean;
  readOnly?: boolean;
};

export type StringField = BaseField & {
  type: "string";
  placeholder?: string;
  options?: FieldOption[];
  pattern?: string;
};

export type TextField = BaseField & {
  type: "text";
  placeholder?: string;
  rows?: number;
};

export type RichtextField = BaseField & {
  type: "richtext";
  placeholder?: string;
  rows?: number;
};

export type NumberField = BaseField & {
  type: "number";
  placeholder?: string;
  options?: FieldOption[];
  min?: number;
  max?: number;
};

export type BooleanField = BaseField & {
  type: "boolean";
};

export type DateField = BaseField & {
  type: "date";
  min?: number;
  max?: number;
};

export type UrlField = BaseField & {
  type: "url";
  placeholder?: string;
};

export type SlugField = BaseField & {
  type: "slug";
  source?: string;
};

export type ImageField = BaseField & {
  type: "image";
  placeholder?: string;
  accept?: string;
};

export type ReferenceField = BaseField & {
  type: "reference";
  to?: string[];
};

export type ArrayField = BaseField & {
  type: "array";
  of?: ArrayItemType;
};

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

export type TypeDef = {
  name: string;
  fields: Field[];
};

export type Schema = {
  types: TypeDef[];
  storage_configured: boolean;
};

export type Status = "draft" | "published" | "archived";

export type Document = {
  id: string;
  type: string;
  status: Status | string;
  data: Record<string, unknown>;
  created_at: string;
  updated_at: string;
  published_at: string | null;
};

export type Draft = {
  status: Status;
  data: Record<string, unknown>;
};

export type ApiToken = {
  id: string;
  name: string;
  expires_at: string | null;
  last_used_at: string | null;
  created_at: string;
};

export type Media = {
  id: string;
  key: string;
  url: string;
  filename: string;
  content_type: string;
  size: number;
  label: string | null;
  uploaded_by: string | null;
  created_at: string;
};
