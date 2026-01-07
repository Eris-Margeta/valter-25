export interface GlobalConfig {
  company_name: string;
  currency_symbol: string;
  locale: string;
}

export interface CloudField {
  key: string;
  type: string;
  required: boolean;
  options?: string[];
}

export interface CloudDefinition {
  name: string;
  icon: string;
  fields: CloudField[];
}

export interface RelationRule {
  field: string;
  target_cloud: string;
}

export interface AggregationRule {
  name: string;
  path: string;
  target_field: string;
  logic: string;
  filter?: string;
}

export interface IslandDefinition {
  name: string;
  root_path: string;
  meta_file: string;
  relations: RelationRule[];
  aggregations: AggregationRule[];
}

export interface AppConfig {
  GLOBAL: GlobalConfig;
  CLOUDS: CloudDefinition[];
  ISLANDS: IslandDefinition[];
}

export interface PendingAction {
  id: string;
  type: string;
  target_table: string;
  key_field: string;
  value: string;
  context: string;
  suggestions: string[];
  status: string;
  created_at: string;
}

export type ConfigStatus =
  | { type: 'compileTime' }
  | { type: 'compileTimeIgnored' }
  | { type: 'runtime' }
  | { type: 'runtimeError'; missingKeys: string[] };meError'; missingKeys: string[] };
