import json
from pathlib import Path

data = json.loads(Path('graphify-out/graph.json').read_text(encoding='utf-8'))

# Build node lookup
nodes_by_id = {n['id']: n for n in data['nodes']}
node_labels = {n['id']: n.get('label', n['id']) for n in data['nodes']}

print("=== Nœuds clés ===")
for nid, n in nodes_by_id.items():
    label = n.get('label', nid)
    if any(kw in label.lower() or kw in nid.lower() for kw in ['module', 'activite', 'adhesion', 'personne', 'cadence']):
        com = n.get('community', '?')
        src = n.get('source_file', '')[-60:]
        print(f'  {nid} -> "{label}" [com={com}] {src}')

print("\n=== Liens vers/depuis Module Activités ===")
for l in data['links']:
    s = l['source']
    t = l['target']
    s_label = node_labels.get(s, s)
    t_label = node_labels.get(t, t)
    rel = l.get('relation', '?')
    conf = l.get('confidence', '?')
    src = l.get('source_file', '')[-60:]
    is_module = 'module_activites' in s.lower() or 'module_activites' in t.lower()
    is_bridge = any(kw in s.lower() for kw in ['adhesion', 'personne']) and any(kw in t.lower() for kw in ['activite'])
    is_bridge = is_bridge or (any(kw in t.lower() for kw in ['adhesion', 'personne']) and any(kw in s.lower() for kw in ['activite']))
    if is_module or is_bridge:
        print(f'  "{s_label}" --[{rel}/{conf}]--> "{t_label}"')
        if src:
            print(f'    source: {src}')