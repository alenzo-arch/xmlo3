from xmlo3 import parse_file_roxmltree, parse_file_xmlparser
from time import perf_counter
from xml.parsers.expat import ParserCreate
from lxml import etree, sax


# # roxmltree
# start = perf_counter()
# parse_file_roxmltree("files/nasa.xml")
# print(f"roxmltree: {perf_counter()- start}")

# xml parser
start = perf_counter()
parse_file_xmlparser("files/nasa.xml")
print(f"xmlparser: {perf_counter()- start}")


# xpat
start = perf_counter()
parser = ParserCreate()
with open("files/nasa.xml") as file:
    doc = file.read()
    parser.Parse(doc)
print(f"expat: {perf_counter()- start}")


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