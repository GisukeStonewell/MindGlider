"""Check mm headers with the actual Freeplane MapVersionInterpreter Java source.
Requires the sibling source clone and a JDK; does not change the clone or launch a GUI.
Usage: python scripts/verify_freeplane_compat.py original.mm saved.mm
"""
from pathlib import Path
import subprocess,sys
base=Path(__file__).resolve().parents[1];up=base.parent/'freeplane/freeplane/src';out=base/'artifacts/compat-0.9/java';out.mkdir(parents=True,exist_ok=True)
stubs={
'org/freeplane/core/extension/IExtension.java':'package org.freeplane.core.extension; public interface IExtension {}',
'org/freeplane/core/resources/ResourceController.java':'''package org.freeplane.core.resources; public class ResourceController { public static ResourceController getResourceController(){return new ResourceController();} public java.net.URL getResource(String name){try{return java.nio.file.Path.of(System.getProperty("freeplane.resources"),name.substring(1)).toUri().toURL();}catch(Exception e){throw new RuntimeException(e);}} }''',
'org/freeplane/core/util/LogUtils.java':'package org.freeplane.core.util; public class LogUtils {public static void severe(Object value){throw new RuntimeException(String.valueOf(value));}}',
'org/freeplane/core/util/TextUtils.java':'package org.freeplane.core.util; public class TextUtils {public static String format(String key,Object... values){return key;} public static String getText(String key){return key;}}',
'org/freeplane/features/map/MapModel.java':'package org.freeplane.features.map; public class MapModel {public <T> T getExtension(Class<T> kind){return null;}}',
'org/freeplane/features/url/IMapConverter.java':'package org.freeplane.features.url; public interface IMapConverter {}',
'CheckMap.java':'''import java.nio.file.*;import java.nio.charset.StandardCharsets;import java.io.*;import java.util.*;import javax.xml.parsers.*;import org.w3c.dom.*;import org.freeplane.features.url.MapVersionInterpreter;
public class CheckMap {
 static Map<String,String> content(Path p)throws Exception {var reader=new InputStreamReader(Files.newInputStream(p),StandardCharsets.UTF_8);var d=DocumentBuilderFactory.newInstance().newDocumentBuilder().parse(new org.xml.sax.InputSource(reader));var nodes=d.getElementsByTagName("node");var result=new TreeMap<String,String>();for(int i=0;i<nodes.getLength();i++){var n=(Element)nodes.item(i);result.put(n.getAttribute("ID"),n.getAttribute("TEXT")+"|"+n.getTextContent());}return result;}
 public static void main(String[] args)throws Exception {var path=Path.of(args[1]);byte[] prefix=new byte[1000];try(var stream=Files.newInputStream(path)){stream.read(prefix);}var dialect=MapVersionInterpreter.getVersionInterpreter(new String(prefix,StandardCharsets.UTF_8));if(dialect.anotherDialect||dialect.needsConversion)throw new AssertionError("Unexpected Freeplane warning/conversion: "+dialect.name);if(!content(Path.of(args[0])).equals(content(path)))throw new AssertionError("Node text or rich content changed");System.out.println("Freeplane source check: "+dialect.name+", warning=false, conversion=false; Unicode node/rich text preserved.");}
}'''
}
files=[]
for name,text in stubs.items():
 p=out/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(text,encoding='utf-8');files.append(str(p))
for name in ['org/freeplane/features/url/MapVersionInterpreter.java','org/freeplane/features/url/IMapInputStreamConverter.java','org/freeplane/core/util/XsltPipeReaderFactory.java']:files.append(str(up/'main/java'/name))
subprocess.run(['javac','-encoding','UTF-8','-d',str(out),*files],check=True)
subprocess.run(['java',f'-Dfreeplane.resources={up/"editor/resources"}','-cp',str(out),'CheckMap',*map(str,map(Path,sys.argv[1:]))],check=True)
