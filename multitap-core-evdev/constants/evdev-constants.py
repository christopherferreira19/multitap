#! /usr/bin/python3
# Parses linux/input.h scanning for #define KEY_FOO 134
# Prints Rust source header files that can be used for
# mapping and lookup tables.
#
# The original version of this file is in libevdev
#

from __future__ import print_function
import re
import sys

CONSTANT_PREFIXES = [
    "EV",
    "SYN",
    "KEY",
    "BTN",
    "REL",
    "ABS",
    "MSC",
    "SW",
    "LED",
    "SND",
    "REP",
    "FF_STATUS",
    "FF",
    "INPUT_PROP",
    "BUS"
]

CONSTANT_MATCHER = re.compile(r"^#define\s+({})_(\w+)\s+(0x[0-9A-Fa-f]+|[0-9]+)"
        .format("|".join(CONSTANT_PREFIXES)))

CONSTANT_BLACKLIST = [
    "EV_VERSION",
    "KEY_RESERVED",
    "BTN_MISC",
    "BTN_MOUSE",
    "BTN_JOYSTICK",
    "BTN_GAMEPAD",
    "BTN_DIGI",
    "BTN_WHEEL",
    "BTN_TRIGGER_HAPPY",
    "REL_RESERVED",
    "ABS_RESERVED",
    "SW_MAX",
    "REP_MAX",
]

TYPE_NAMES = {
    "EV": "Type",
    "SYN": "SyncKind",
    "KEY": "KeyId",
    "BTN": "KeyId",
    "ABS": "AxisId",
    "REL": "MotionId",
    "FF": "FF",
    "FF_STATUS": "FFStatus",
}

class Constant:
    @staticmethod
    def type_name(constant_name):
        if constant_name in TYPE_NAMES:
            return TYPE_NAMES[constant_name]
        components = constant_name.split('_')
        return ''.join(x.title() for x in components)
    def __init__(self, match):
        self._prefix = match[1]
        self.group_name = Constant.type_name(self._prefix)
        self.subname = match[2]
        if match[3].startswith("0x"):
            self.value = int(match[3][2:], 16)
        else:
            self.value = int(match[3], 10)

    @property
    def name(self):
        return "{}_{}".format(self._prefix, self.subname)

class Group:
    def __init__(self, name):
        self.name = name
        self.type = None
        self.max = None
        self._values = {}
        self.done = False
    def add(self, constant):
        self._values[constant.value] = constant
    def find_by_name(self, name):
        for constant in self:
            if constant.name == name:
                return constant
    def __iter__(self):
        return self._values.values().__iter__()

class Parser:
    def parse(self, fp, groups):
        lines = fp.readlines()
        for line in lines:
            self._parse_line(groups, line)
    def _parse_line(self, groups, line):
        match = CONSTANT_MATCHER.match(line)
        if match == None:
            return

        constant = Constant(match)
        if constant.name in CONSTANT_BLACKLIST:
            return

        group_name = constant.group_name
        if not group_name in groups:
            groups[group_name] = Group(group_name)
        group = groups[group_name]
        if constant.subname == "MAX":
            group.max = constant
        else:
            group.add(constant)

class Printer:
    def print_header(self):
        print("/* THIS FILE IS GENERATED, DO NOT EDIT */")
        print("")
    def print_group(self, group, impl_input_id = False):
        if group is None:
            return
        name = group.name
        print( "#[derive(Copy, Clone, PartialEq, Eq, Serialize)]")
        if not impl_input_id:
            print( "#[derive(Debug, Deserialize)]")
        print( "#[repr(C)]")
        print(f"pub struct {name}(pub __u16);")
        if impl_input_id:
            self._print_group_impl_input_id(group, name)
        print(f"impl {name} {{")
        if group.max is not None:
            print(f"    pub fn max() -> Self {{ {name}({hex(group.max.value)}) }}")
        for constant in group:
            print(f"    pub const {constant.name}: {name} = {name}({hex(constant.value)});")
        print("}")
        print("")
        group.done = True
    def _print_group_impl_input_id(self, group, name):
        print(f"impl InputId for {name} {{")
        print(f"    const TYPE_CODE: c_uint = {hex(group.type.value)};")
        print(f"    const TYPE_NAME: &'static str = \"{name}\";")
        print( "    fn from_raw(raw: __u16) -> Self { Self(raw) }")
        print( "    fn as_raw(&self) -> __u16 { self.0 }")
        print("}")
        print(f"impl fmt::Debug   for {name} {{ fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {{ self.input_id_debug(f)   }} }}")
        print(f"impl fmt::Display for {name} {{ fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {{ self.input_id_display(f) }} }}")
        print(f"impl<'de> serde::de::Deserialize<'de> for {name} {{")
        print(f"    fn deserialize<D>(deserializer: D) -> Result<{name}, D::Error>")
        print( "    where")
        print( "        D: Deserializer<'de>,")
        print( "    {")
        print( "        deserializer.deserialize_any(InputIdDeserializeVisitor::new())")
        print( "    }")
        print( "}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} /path/to/linux/header.h...")
        sys.exit(2)

    groups = dict()
    parser = Parser()
    for arg in sys.argv[2:]:
        with open(arg) as file:
            parser.parse(file, groups)

    for constant in groups['Type']:
        type_name = Constant.type_name(constant.subname)
        if type_name in groups:
            groups[type_name].type = constant
        elif not type_name in ["Pwr"]:
            print(f"Unknown Type {type_name}", file=sys.stderr)

    printer = Printer()
    printer.print_header()
    types = groups['Type']
    printer.print_group(groups['SyncKind'], impl_input_id = True)
    printer.print_group(groups['KeyId'], impl_input_id = True)
    printer.print_group(groups['AxisId'], impl_input_id = True)
    printer.print_group(groups['MotionId'], impl_input_id = True)
        