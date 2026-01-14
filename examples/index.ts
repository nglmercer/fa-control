import { demonstrateGetters } from './getters';
import { demonstrateSetters } from './setters';

async function main() {
  await demonstrateGetters();
  await demonstrateSetters();
}

main().catch((error) => console.error({ error }));
