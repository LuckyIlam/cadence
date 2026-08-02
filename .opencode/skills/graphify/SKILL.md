---
name: graphify
description: Utilise le graphe de connaissances (graphify) pour répondre aux questions sur le code, explorer les relations entre concepts et maintenir le graphe à jour après chaque modification.
---

## Mission

Donner accès au graphe de connaissances du projet situé dans `graphify-out/` : répondre aux questions sur le code via des sous-graphes ciblés plutôt que de fouiller les fichiers bruts, et maintenir le graphe cohérent avec l'état réel du code et de la documentation.

> Skill partagé entre **Cline** (via la jonction `.claude/skills` → `.opencode/skills`) et **OpenCode** : un seul fichier source sert aux deux assistants.

## Fichiers

| Fichier | Rôle |
|---------|------|
| `graphify-out/graph.json` | Graphe de connaissances (données : nœuds, arêtes, communautés) |
| `graphify-out/GRAPH_REPORT.md` | Résumé : hubs, god nodes, communautés, écarts de connaissance |
| `graphify-out/graph.html` | Visualisation interactive (vis-network) |
| `.graphifyignore` | Fichiers/dossiers exclus du graphe (schémas générés, `node_modules`, …) |

## Commandes

- `graphify query "<question>"` — réponse ciblée sur un nœud et ses relations (parcours BFS/DFS du sous-graphe).
- `graphify explain "<concept>"` — explication en clair d'un nœud et de ses voisins.
- `graphify path "<A>" "<B>"` — plus court chemin entre deux nœuds.
- `graphify update .` — rafraîchit le graphe (extraction AST seule, sans LLM) après modification du code ou de la doc.
- `graphify tree` / `graphify export callflow-html` — visualisations complémentaires optionnelles.

## Règles

- Tester `graphify query` / `explain` / `path` **avant** de lire les fichiers bruts (grep).
- Lire `GRAPH_REPORT.md` seulement pour une vue globale de l'architecture.
- ⚠ Aucune clé LLM n'est configurée dans l'environnement : n'utiliser que `graphify update .` (hors-ligne), jamais `graphify extract` sémantique (échec sans clé).
- Après toute modification de code ou de documentation, lancer `graphify update .` pour garder le graphe à jour.
- La commande `graphify` est sur le PATH (sinon : `C:\Users\Eric\dev\graphify\bin\graphify.exe`).
