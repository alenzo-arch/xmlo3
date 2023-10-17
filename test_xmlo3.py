from time import perf_counter
from typing import Annotated
from xml.parsers.expat import ParserCreate

from pydantic import StringConstraints
from pydantic_xml import BaseXmlModel, attr, element
from rich import print

from archfx.s1.core.xml_parser import parse_xml
from xmlo3 import parse_file, parse_string


class Nest2(BaseXmlModel, tag="nest2"):
    attr: int = attr()


class Nest1(BaseXmlModel, tag="nest1"):
    message: Annotated[str, StringConstraints(strip_whitespace=True)]
    nest2: Nest2


class Note(BaseXmlModel, tag="note"):
    id: int = attr()
    to: str = element(tag="to")
    sender: str = element(tag="from")
    heading: str = element(tag="heading")
    nest1: Nest1
    body: str = element()


# xml parser
start = perf_counter()
out = tuple(parse_file("files/note.xml"))
print(f"parse_file: {perf_counter()- start}")


start = perf_counter()
parser = ParserCreate()
with open("files/note.xml") as file:
    out = tuple(parse_string(file.read()))
print(f"parse_string: {perf_counter()- start}")

# expat
start = perf_counter()
parser = ParserCreate()
with open("files/note.xml") as file:
    out = parse_xml(file.read().encode())
print(f"expat: {perf_counter()- start}")


with open("files/note.xml") as file:
    out = Note.from_xml(file.read().encode())
print(f"pydantic-xml: {perf_counter()- start}")
print(out)


def test_parse():
    with open("files/note.xml") as file:
        input = file.read()
        rust = tuple(parse_string(input))
        print(rust)
        python = parse_xml(input.encode())
        print(python)
        assert rust == python


# # lxml
# start = perf_counter()

# tree = etree.parse("files/note.xml")
# print(f"lxml tree: {perf_counter()- start}")

# start = perf_counter()
# with open("files/note.xml") as file:
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
