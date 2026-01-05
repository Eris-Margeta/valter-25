// Osiguravamo da su svi tipovi exportani
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
  logic: string; // 'sum', 'count', 'average'
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
  global: GlobalConfig;
  clouds: CloudDefinition[];
  islands: IslandDefinition[];
}

// OVO JE ONO ŠTO JE NEDOSTAJALO ILI BILO KEŠIRANO KRIVO
export interface PendingAction {
  id: string;
  type: string; // npr. 'CreateEntity'
  target_table: string; // npr. 'Klijent'
  key_field: string; // npr. 'naziv'
  value: string; // npr. 'Mircosoft'
  context: string; // npr. 'Pronađeno u projektu: Projekt Phoenix'
  suggestions: string[]; // niz sličnih imena
  status: string; // 'Pending', 'Resolved', 'Rejected'
  created_at: string;
}

