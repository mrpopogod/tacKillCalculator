# tacKillCalculator

Basic CLI utility to take an SSW mech file and calculate how likely it is to die when a TAC is rolled for both basic rules (TACs hit the CT only) and floating crits (location of TAC is rerolled).

Note: At this point in time the xml parsing library requires an xmlns attribute to exist on the root \<mech\> element; simply add xmlns="" and it can parse the SSW file.
