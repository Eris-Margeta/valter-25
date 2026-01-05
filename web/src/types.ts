
// Definicije tipova koje odgovaraju Rust strukturama iz config.rs

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

export interface AggregationRule {
  name: string;
  logic: string;
}

export interface IslandDefinition {
  name: string;
  root_path: string;
  aggregations: AggregationRule[];
}

export interface AppConfig {
  GLOBAL: GlobalConfig;
  CLOUDS: CloudDefinition[];
  ISLANDS: IslandDefinition[];
}

export interface PendingAction {
  id: string;
  target_table: string;
  value: string;
  context: string;
  suggestions: string[]; // JSON array string
}
