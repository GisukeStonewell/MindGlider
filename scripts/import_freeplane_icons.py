"""Import Freeplane SVGs without renaming BUILTIN IDs; flatten gradients/shadows.
Original SVGs and their metadata are retained for attribution and reproducibility.
Run from mindglider: python scripts/import_freeplane_icons.py
"""
from pathlib import Path
import xml.etree.ElementTree as ET
import re,json,hashlib,shutil
base=Path(__file__).resolve().parents[1]; source=base.parent/'freeplane/freeplane/src/viewer/resources/images/icons'
out=base/'assets/freeplane-icons'; ET.register_namespace('', 'http://www.w3.org/2000/svg'); ET.register_namespace('xlink','http://www.w3.org/1999/xlink')
files=sorted(source.rglob('*.svg')); rows=[]; manifest=[]
# Preserve the standard group order from Freeplane's registration properties.
props=(source.parents[1]/'freeplane.properties').read_text(encoding='utf-8').replace('\\\n','')
properties=dict(line.split('=',1) for line in props.splitlines() if '=' in line and not line.startswith('#'))
order=[]
def walk(group):
 order.extend(properties.get('icons.group.'+group,'').split(';'))
 for nested in properties.get('icons.groups.'+group,'').split(';'):
  if nested:walk(nested)
for group in properties['icons.groups'].split(';'):walk(group)
files.sort(key=lambda p:(order.index(p.relative_to(source).as_posix()[:-4]) if p.relative_to(source).as_posix()[:-4] in order else 10000,p.as_posix()))
for p in files:
 name=p.relative_to(source).as_posix()[:-4]; original=p.read_bytes(); root=ET.fromstring(original); flat=out/'simplified'/p.relative_to(source); orig=out/'original'/p.relative_to(source)
 orig.parent.mkdir(parents=True,exist_ok=True);orig.write_bytes(original)
 nodes={e.get('id'):e for e in root.iter() if e.get('id')}
 def gradient_color(e,seen=()):
  if e.get('id') in seen:return '#808080'
  stops=list(e.findall('{*}stop'))
  if not stops:
   ref=e.get('{http://www.w3.org/1999/xlink}href','').lstrip('#');return gradient_color(nodes[ref],seen+(e.get('id'),)) if ref in nodes else '#808080'
  colors=[]
  for stop in stops:
   c=stop.get('stop-color') or next((x.split(':',1)[1] for x in stop.get('style','').split(';') if x.startswith('stop-color:')),'#808080')
   if re.fullmatch(r'#[0-9a-fA-F]{3}',c):c='#'+''.join(x*2 for x in c[1:])
   if re.fullmatch(r'#[0-9a-fA-F]{6}',c):colors.append(tuple(int(c[i:i+2],16) for i in (1,3,5)))
  if not colors:return '#808080'
  return '#'+''.join(f'{round(sum(c[i] for c in colors)/len(colors)):02x}' for i in range(3))
 def transparent(e,seen=()):
  if e.get('id') in seen:return False
  for stop in e.findall('{*}stop'):
   alpha=stop.get('stop-opacity') or next((x.split(':',1)[1] for x in stop.get('style','').split(';') if x.startswith('stop-opacity:')),'1')
   try:
    if float(alpha)<1:return True
   except ValueError:pass
  ref=e.get('{http://www.w3.org/1999/xlink}href','').lstrip('#')
  return transparent(nodes[ref],seen+(e.get('id'),)) if ref in nodes else False
 gradients={i:gradient_color(e) for i,e in nodes.items() if e.tag.endswith(('linearGradient','radialGradient')) and not transparent(e)}
 prefix='sp'+hashlib.sha256(name.encode()).hexdigest()[:12]+'-'
 for parent in list(root.iter()):
  for child in list(parent):
   if child.tag.rsplit('}',1)[-1] in ['metadata','namedview','filter']:parent.remove(child)
  for key,value in list(parent.attrib.items()):
   if key.startswith('{') and not key.startswith('{http://www.w3.org/1999/xlink}'):
    del parent.attrib[key];continue
   if key=='filter':del parent.attrib[key];continue
   if key=='id':parent.set(key,prefix+value);continue
   value=re.sub(r'url\(#([^)]*)\)',lambda m:gradients.get(m[1],'url(#'+prefix+m[1]+')'),value)
   if key.endswith('href') and value.startswith('#'):value='#'+prefix+value[1:]
   if key=='style':value=re.sub(r'filter\s*:[^;]+;?','',value)
   parent.set(key,value)
 if 'viewBox' not in root.attrib:
  w=re.sub('[^0-9.]','',root.get('width','24'));h=re.sub('[^0-9.]','',root.get('height','24'));root.set('viewBox',f'0 0 {w} {h}')
 root.set('width','24');root.set('height','24');flat.parent.mkdir(parents=True,exist_ok=True);flat.write_text(ET.tostring(root,encoding='unicode'),encoding='utf-8')
 rows.append(f'    Icon {{ id: {json.dumps(name)}, key: {json.dumps("icon."+name)}, image: include_str!("../assets/freeplane-icons/simplified/{p.relative_to(source).as_posix()}") }},')
 manifest.append({'id':name,'source':p.relative_to(base.parent/'freeplane').as_posix(),'sha256':hashlib.sha256(original).hexdigest()})
(out/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
shutil.copyfile(base.parent/'freeplane/license.txt',out/'Freeplane-LICENSE.txt')
code='''//! Simplified Freeplane resource images. See assets/freeplane-icons/README.md.
pub struct Icon { pub id: &'static str, pub key: &'static str, pub image: &'static str }
pub static CATALOG: &[Icon] = &[
'''+ '\n'.join(rows)+'''
];
pub fn find(name: &str) -> Option<&'static Icon> { CATALOG.iter().find(|i| i.id == name) }
pub fn body(name: &str) -> String {
    if let Some(icon) = find(name) { return icon.image.into(); }
    if let Some(progress) = name.strip_suffix('%').and_then(|p| p.parse::<u8>().ok()).filter(|p| *p <= 100) {
        return format!("<g fill=\\"none\\" stroke=\\"#506780\\"><circle cx=\\"12\\" cy=\\"12\\" r=\\"9\\" stroke-width=\\"2\\" stroke-opacity=\\".25\\"/><circle cx=\\"12\\" cy=\\"12\\" r=\\"9\\" stroke-width=\\"4\\" stroke-dasharray=\\"{} 56.55\\" transform=\\"rotate(-90 12 12)\\"/></g>",56.55*progress as f32/100.);
    }
    find("help").unwrap().image.into()
}
pub fn svg(name: &str) -> String { format!("<svg xmlns=\\"http://www.w3.org/2000/svg\\" width=\\"24\\" height=\\"24\\" viewBox=\\"0 0 24 24\\">{}</svg>",body(name)) }
/// Toggle a palette icon atomically through the caller's History transaction.
pub fn toggle(doc: &mut crate::model::Document, targets: &std::collections::BTreeSet<String>, id: &str) -> anyhow::Result<()> {
    anyhow::ensure!(find(id).is_some(), "Unknown built-in icon ID");
    for target in targets { doc.ensure_editable(target)?; }
    for target in targets {
        let node=doc.nodes.get_mut(target).unwrap();
        if let Some(p)=id.strip_suffix('%').and_then(|s|s.parse::<u8>().ok()) {
            node.progress=if node.progress==Some(p) {None} else {Some(p)};
        } else if node.icons.iter().any(|i|i==id) { node.icons.retain(|i|i!=id); }
        else { node.icons.push(id.into()); }
    }
    Ok(())
}
#[cfg(test)] mod tests {
 #[test] fn every_imported_icon_is_unique_and_renderable() {
  let mut ids=std::collections::BTreeSet::new();
  for icon in super::CATALOG {
   assert!(ids.insert(icon.id));
   let tree=resvg::usvg::Tree::from_str(&super::svg(icon.id), &Default::default()).unwrap();
   assert!(!tree.root().children().is_empty(), "{}",icon.id);
  }
 }
}
'''
(base/'src/icons.rs').write_text(code,encoding='utf-8')
for language in ['ja','en']:
 p=base/'locales'/f'{language}.json';d=json.loads(p.read_text(encoding='utf-8'))
 translation=base.parent/'freeplane/freeplane/src'/('editor/resources/translations/Resources_ja.properties' if language=='ja' else 'viewer/resources/translations/Resources_en.properties')
 labels={}
 for line in translation.read_text(encoding='latin-1').splitlines():
  if line.startswith('icon_') and '=' in line:
   key,value=line.split('=',1);labels[key[5:]]=re.sub(r'\\u([0-9a-fA-F]{4})',lambda m:chr(int(m[1],16)),value)
 for item in manifest:d['icon.'+item['id']]=labels.get(item['id'],item['id'].replace('links/','').replace('_',' '))
 p.write_text(json.dumps(d,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print('Imported',len(files),'Freeplane SVG icons')
