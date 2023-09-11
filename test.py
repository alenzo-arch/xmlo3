from time import perf_counter
from xml.parsers.expat import ParserCreate

from archfx.s1.core.xml_parser import parse_xml
from xmlo3 import parse_file, parse_string

# # roxmltree
# start = perf_counter()
# parse_file_roxmltree("files/nasa.xml")
# print(f"roxmltree: {perf_counter()- start}")

# xml parser
start = perf_counter()
out = parse_file("files/nasa.xml")
print(f"parse_file: {perf_counter()- start}")


start = perf_counter()
parser = ParserCreate()
with open("files/nasa.xml") as file:
    out = parse_string(file.read())
print(f"parse_string: {perf_counter()- start}")

# xpat
start = perf_counter()
parser = ParserCreate()
with open("files/nasa.xml") as file:
    out = parse_xml(file.read())
print(f"expat: {perf_counter()- start}")


with open("files/note.xml") as file:
    input = file.read()
    rust = parse_string(input)
    python = parse_xml(input)
    assert rust == python

# # lxml
# start = perf_counter()

# tree = etree.parse("files/nasa.xml")
# print(f"lxml tree: {perf_counter()- start}")

# start = perf_counter()
# with open("files/nasa.xml") as file:
#     parser = etree.XMLPullParser()
#     doc = file.read()
#     parser.feed(doc)
#     parser.close()
# print(f"lxml pull: {perf_counter()- start}")


# # xml parser
# start = perf_counter()
# out = parse_file_xmlparser("files/note.xml")
# print(out)
# print(f"xmlparser: {perf_counter()- start}")
