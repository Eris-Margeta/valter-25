// Detekcija okoline
const isDev = import.meta.env.DEV;

// U Devu (Vite 5173) gađamo 8000.
// U Prod (Rust 9090) koristimo relativni path (isti host/port).
const BASE_URL = isDev ? 'http://localhost:8000' : '';
const GRAPHQL_URL = `${BASE_URL}/graphql`;

console.log(`[VALTER LINK] Mode: ${isDev ? 'DEV' : 'PROD'}`);
console.log(`[VALTER LINK] Connecting to: ${GRAPHQL_URL}`);

export async function graphqlRequest(query: string, variables: any = {}) {
  try {
    const res = await fetch(GRAPHQL_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ query, variables })
    });
    
    if (!res.ok) {
      const txt = await res.text();
      console.error("[VALTER LINK] HTTP Error:", res.status, txt);
      throw new Error(`Server Error (${res.status})`);
    }

    const json = await res.json();
    if (json.errors) {
      console.warn("[VALTER LINK] GraphQL Error:", json.errors);
      throw new Error(json.errors[0].message);
    }
    return json.data;
  } catch (e) {
    console.error("[VALTER LINK] Network/Parse Error:", e);
    throw e;
  }
}

export const QUERIES = {
  GET_CONFIG: `
    query { 
      config
    }
  `,
  GET_CLOUD_DATA: `
    query($name: String!) { 
      cloudData(name: $name) 
    }
  `,
  GET_ISLAND_DATA: `
    query($name: String!) {
      islandData(name: $name)
    }
  `,
  GET_PENDING_ACTIONS: `
    query {
      pendingActions
    }
  `,
  ASK_ORACLE: `
    query($q: String!) {
      askOracle(question: $q)
    }
  `
};

export const MUTATIONS = {
  // NEW: Rescan mutation
  RESCAN_ISLANDS: `
    mutation {
      rescanIslands
    }
  `,
  RESOLVE_ACTION: `
    mutation($id: String!, $choice: String!) {
      resolveAction(actionId: $id, choice: $choice)
    }
  `,
  UPDATE_ISLAND_FIELD: `
    mutation($type: String!, $name: String!, $key: String!, $value: String!) {
      updateIslandField(islandType: $type, islandName: $name, key: $key, value: $value)
    }
  `
};
