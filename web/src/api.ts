const API_URL = 'http://localhost:8000/graphql';

export async function graphqlRequest(query: string, variables: any = {}) {
  const res = await fetch(API_URL, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ query, variables })
  });
  const json = await res.json();
  if (json.errors) {
    throw new Error(json.errors[0].message);
  }
  return json.data;
}

export const QUERIES = {
  GET_CONFIG: `
    query { 
      config {
        global {
          company_name
          currency_symbol
          locale
        }
        clouds {
          name
          icon
          fields {
            key
            type
            required
            options
          }
        }
        islands {
          name
          root_path
          meta_file
          relations {
            field
            target_cloud
          }
          aggregations {
            name
            path
            target_field
            logic
            filter
          }
        }
      }
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
      pendingActions {
        id
        type
        target_table
        key_field
        value
        context
        suggestions
        status
        created_at
      }
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
  `
};

