#!/usr/bin/env python3
# very VERY simple python tool to help add commands
import json

with open("commands.json", 'r') as file:
    d = json.read(file)

"""
"bnb": {
  "value": {
    "StringResponse": "Brand new bot!"
  },
  "admin_only": false,
  "subcommands": {"sc":{"value":{"StringResponse":"Yes, we got subcommands!"}}},
  "hidden": true
},
"""

def to_bool(string):
    return string != '' and (string.lower()[0] in ['y', 't', '1'])

while True:
    name = input('command name: ').strip()
    if name in d:
        print('already done')
        continue
    q = {}

    y = input('generic or string resp? empty for string: ').strip()
    if y == '':
        resp = input('What is the response? Empty cancels: ').strip()
        if resp == '':
            continue
        q["value"] = {"StringResponse": resp}
    else:
        resp = input('What is the mapping? Empty cancels: ').strip()
        if resp == '':
            continue
        q["value"] = {"Generic": resp}

    q["admin_only"] = to_bool(input("admin only? y for true, otherwise false: ").strip())
    q["hidden"] = to_bool(input("hidden? y for true, otherwise false: ").strip())

    d[name] == q

    if input('empty to go again: ').strip() != '':
        break

print('committing to file')
with open('commands.json', 'w') as file:
    json.dump(d, file, indent=4)
