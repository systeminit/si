# Council

## Technical Debts

- Multiple AttributeValues may be created for the same slot across change-sets
- Array slots elements created in multiple change-sets are problematic (should we merge, choose one, etc...)
- If processing a single attribute value fails in pinga we should be able to keep processing other attribute values that are not dependent on it
   - Right now we halt the entire job and remove it from the graph, we should only remove the hold on the problematic branch
