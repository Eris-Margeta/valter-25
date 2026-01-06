// Detektira jesmo li u Vite Dev modu
const isDev = import.meta.env.DEV;

// U devu (Vite radi na 5173) moramo eksplicitno gađati Backend na 8000.
// U produkciji (Rust servira i HTML i API), API je na istom hostu/portu, pa koristimo relativnu putanju.
const API_URL = isDev ? 'http://localhost:8000/graphql' : '/graphql';

export async function graphqlRequest(query: string, variables: any = {}) {
  try {
    const res = await fetch(API_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ query, variables })
    });
    
    if (!res.ok) {
      throw new Error(`HTTP Error: ${res.status}`);
    }

    const json = await res.json();
    if (json.errors) {
      throw new Error(json.errors[0].message);
    }
    return json.data;
  } catch (e) {
    console.error("API Call Failed:", e);
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

