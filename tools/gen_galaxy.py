#!/usr/bin/env python3
"""Generate elite-core/galaxies/galaxy1.json from the HYG star database.

Usage: tools/gen_galaxy.py hyg_v44.csv > elite-core/galaxies/galaxy1.json
Data: https://codeberg.org/astronexus/hyg (data/hyg/CURRENT/hyg_v44.csv.gz), CC BY-SA 4.0.
"""
import csv
import json
import math
import random
import re
import sys
from pathlib import Path

SEED = 1984
SYSTEMS = 256
GRID = 255
MIN_SEPARATION = 4
JUMP_RANGE = 7
MIN_NEIGHBOURS = 2

GREEK = {
    "Alp": "Alpha", "Bet": "Beta", "Gam": "Gamma", "Del": "Delta", "Eps": "Epsilon",
    "Zet": "Zeta", "Eta": "Eta", "The": "Theta", "Iot": "Iota", "Kap": "Kappa",
    "Lam": "Lambda", "Mu": "Mu", "Nu": "Nu", "Xi": "Xi", "Omi": "Omicron", "Pi": "Pi",
    "Rho": "Rho", "Sig": "Sigma", "Tau": "Tau", "Ups": "Upsilon", "Phi": "Phi",
    "Chi": "Chi", "Psi": "Psi", "Ome": "Omega",
}

GENITIVE = {
    "And": "Andromedae", "Ant": "Antliae", "Aps": "Apodis", "Aqr": "Aquarii",
    "Aql": "Aquilae", "Ara": "Arae", "Ari": "Arietis", "Aur": "Aurigae",
    "Boo": "Bootis", "Cae": "Caeli", "Cam": "Camelopardalis", "Cnc": "Cancri",
    "CVn": "Canum Venaticorum", "CMa": "Canis Majoris", "CMi": "Canis Minoris",
    "Cap": "Capricorni", "Car": "Carinae", "Cas": "Cassiopeiae", "Cen": "Centauri",
    "Cep": "Cephei", "Cet": "Ceti", "Cha": "Chamaeleontis", "Cir": "Circini",
    "Col": "Columbae", "Com": "Comae Berenices", "CrA": "Coronae Australis",
    "CrB": "Coronae Borealis", "Crv": "Corvi", "Crt": "Crateris", "Cru": "Crucis",
    "Cyg": "Cygni", "Del": "Delphini", "Dor": "Doradus", "Dra": "Draconis",
    "Equ": "Equulei", "Eri": "Eridani", "For": "Fornacis", "Gem": "Geminorum",
    "Gru": "Gruis", "Her": "Herculis", "Hor": "Horologii", "Hya": "Hydrae",
    "Hyi": "Hydri", "Ind": "Indi", "Lac": "Lacertae", "Leo": "Leonis",
    "LMi": "Leonis Minoris", "Lep": "Leporis", "Lib": "Librae", "Lup": "Lupi",
    "Lyn": "Lyncis", "Lyr": "Lyrae", "Men": "Mensae", "Mic": "Microscopii",
    "Mon": "Monocerotis", "Mus": "Muscae", "Nor": "Normae", "Oct": "Octantis",
    "Oph": "Ophiuchi", "Ori": "Orionis", "Pav": "Pavonis", "Peg": "Pegasi",
    "Per": "Persei", "Phe": "Phoenicis", "Pic": "Pictoris", "Psc": "Piscium",
    "PsA": "Piscis Austrini", "Pup": "Puppis", "Pyx": "Pyxidis", "Ret": "Reticuli",
    "Sge": "Sagittae", "Sgr": "Sagittarii", "Sco": "Scorpii", "Scl": "Sculptoris",
    "Sct": "Scuti", "Ser": "Serpentis", "Sex": "Sextantis", "Tau": "Tauri",
    "Tel": "Telescopii", "Tri": "Trianguli", "TrA": "Trianguli Australis",
    "Tuc": "Tucanae", "UMa": "Ursae Majoris", "UMi": "Ursae Minoris",
    "Vel": "Velorum", "Vir": "Virginis", "Vol": "Volantis", "Vul": "Vulpeculae",
}

OVERRIDES = {"Rigil Kentaurus": "Alpha Centauri"}

def bayer_name(bf, con):
    rest = bf.strip()
    if con and rest.endswith(con):
        rest = rest[: -len(con)].strip()
    match = re.fullmatch(r"(\d+)?\s*([A-Za-z]{3})?(\d)?", rest)
    if not match or not con or con not in GENITIVE:
        return None
    flamsteed, greek, sup = match.groups()
    if greek and greek in GREEK:
        return f"{GREEK[greek]}{sup or ''} {GENITIVE[con]}"
    if flamsteed:
        return f"{flamsteed} {GENITIVE[con]}"
    return None


def catalogue_name(gl):
    gl = re.sub(r"(\d)[A-Z]$", r"\1", gl.strip())
    return re.sub(r"^Gl ", "Gliese ", gl)


def star_name(row):
    proper = re.sub(r" [A-Z]$", "", row["proper"].strip())
    if proper:
        return OVERRIDES.get(proper, proper)
    return bayer_name(row["bf"], row["con"].strip()) or (
        catalogue_name(row["gl"]) if row["gl"].strip() else None
    )


STAR = {
    "O": ["A furious blue giant lights this system with hard ultraviolet.", "A blue-white furnace burns here; ships dock behind heavy radiation shields."],
    "B": ["A blue-white giant floods the system with fierce light.", "A young, blazing star scorches the inner worlds."],
    "A": ["A brilliant white star bathes the system in glare.", "A white star shines here, bright enough to read by from the outer docks."],
    "F": ["A bright yellow-white star warms a busy system.", "A restless yellow-white star lights the lanes."],
    "G": ["A steady yellow star keeps the lanes bright.", "A sun much like Earth's warms the docks here."],
    "K": ["A copper-orange star glows over a quiet system.", "An orange dwarf burns slow and steady here.", "A warm orange sun hangs low over the stations."],
    "M": ["A dim red dwarf casts a ruddy twilight.", "A faint red sun leaves the system in perpetual dusk.", "A small red star flickers sullenly over the docks.", "A flare-prone red dwarf keeps pilots watching their shields."],
    "D": ["A cooling white dwarf, the dead heart of an old star, lights the system.", "The pale ember of a white dwarf is all the light there is."],
    "L": ["A failed star, a brown dwarf, glows a dull red in the dark.", "No true sun here, only a brown dwarf's faint heat."],
}
STAR_FALLBACK = ["A faint, little-charted star holds the system together.", "A dim star few pilots have bothered to chart."]

ECONOMY = {
    "Rich Ind": ["Vast orbital factories turn out machinery night and day.", "Refineries and assembly yards make this one of the richest industrial hubs."],
    "Average Ind": ["Busy foundries and shipyards keep the traffic moving.", "Steady industry hums across the system's stations."],
    "Mainly Ind": ["Smoke-stacked stations and shipyards crowd the orbits.", "Heavy industry dominates, and so does the noise."],
    "Poor Ind": ["Ageing plants limp along on spares and stubbornness.", "Rusting factory decks still turn out a little something."],
    "Rich Agri": ["Lush farm worlds feed half the sector.", "Golden harvests and exotic vintages make this a gourmet's stop."],
    "Average Agri": ["Terraced farm stations ship grain and livestock to hungry neighbours.", "Farming settlements keep a quiet, honest trade."],
    "Mainly Agri": ["Endless fields stretch beneath the orbital docks.", "Almost everything here grows, and everything else is eaten."],
    "Poor Agri": ["Thin soil and tired crops make for hard living.", "Failed harvests have the farmers eyeing every passing freighter."],
}

GOVERNMENT = {
    "Anarchy": ["No one rules here, and pilots dock at their own risk.", "Warlords and gangs run the stations by turns."],
    "Communist": ["The collective runs everything, and the paperwork runs the collective.", "Shared stores and long queues at the dock office."],
    "Confederacy": ["A loose league of stations argues its way through each day.", "Rival stations share a flag and little else."],
    "Corporate State": ["Company logos cover every bulkhead; the shareholders always win.", "The board owns the docks, the air, and the fine print."],
    "Democracy": ["Lively elections keep the station councils on their toes.", "Citizens vote on everything, from tariffs to the docking queue."],
    "Dictatorship": ["A strongman's portrait watches every docking bay.", "One ruler holds the system, and loyalty is not optional."],
    "Feudal": ["Noble houses hold the stations and guard their honour jealously.", "Sworn lords tax each ship that passes their keeps."],
    "Multi-Government": ["Rival factions share the system in an uneasy peace.", "Several governments trade, bicker and spy on one another here."],
}

FAMOUS = {
    "Sol": "The cradle of humanity. Earth's oceans glitter beneath a crowd of orbital docks, and every trade route seems to start here.",
    "Proxima Centauri": "The nearest star to Sol, a small red dwarf prone to violent flares. Its stations hug the one temperate world and keep their shields up.",
    "Alpha Centauri": "A bright pair of suns circling each other, the first stop beyond home. Every traveller remembers the double sunrise.",
    "Barnard's Star": "A swift, faint red dwarf racing across the sky. Prospectors work its cold worlds hoping for a lucky strike.",
    "Wolf 359": "A tiny, flickering red dwarf with a bad reputation and worse flares. Only the stubborn stay.",
    "Sirius": "The brightest star in the sky, blazing white and bold, with a ghostly white dwarf companion. Traders flock to its glittering docks.",
    "Procyon": "A bright yellow-white star trailing a faint white dwarf. A sharp, ambitious port town on the edge of the lanes.",
    "Tau Ceti": "A calm, Sun-like star in a quiet sector, long dreamed of as a second home. Patient settlers have made it one.",
    "Ran": "A young orange star ringed by dusty belts. Prospectors and pirates both love the cover the debris gives them.",
    "61 Cygni": "A pair of orange dwarfs, the first star ever to have its distance measured. Surveyors still come here on pilgrimage.",
    "Epsilon Indi": "A wandering orange star, trailed by a pair of brown dwarfs. Few lights burn out here, and every dock is glad of visitors.",
    "Kapteyn's Star": "An ancient red dwarf from the galaxy's old halo, hurtling through at breakneck speed. Settlers here feel like they are always passing through.",
    "Kruger 60": "Two dim red stars locked in a slow dance, lighting a dusty system with a brooding glow.",
    "Van Maanen's Star": "A lonely white dwarf, the pale remnant of a dead sun. Its cold stations are a haunt for scavengers.",
    "Gliese 65": "A pair of flare stars that can double their brightness in seconds. Traders time their arrivals carefully.",
    "Lalande 21185": "A bright red dwarf with a famous record in the star catalogues. Surveyors, astronomers and smugglers all like the peace.",
    "Luyten's Star": "A quiet red dwarf with a handful of temperate worlds. A good place for those who prefer being left alone.",
    "Groombridge 34": "Two red dwarfs on a long, lazy orbit. The sector's lanes meet here and so does its gossip.",
}


def star_line(row, rng):
    spect = row["spect"].strip()
    letter = spect[:1].upper()
    if spect.lower().startswith("sd"):
        letter = spect[2:3].upper()
    return rng.choice(STAR.get(letter, STAR_FALLBACK))


def description(name, row, entry):
    if name in FAMOUS:
        return FAMOUS[name]
    rng = random.Random(f"{SEED}:{name}")
    return " ".join([
        star_line(row, rng),
        rng.choice(ECONOMY[entry["economy"]["name"]]),
        rng.choice(GOVERNMENT[entry["government"]["name"]]),
    ])


def main(path):
    traits = json.loads((Path(__file__).parent / "galaxy_traits.json").read_text())
    rows = [r for r in csv.DictReader(open(path)) if r["comp_primary"] in ("", r["id"])]
    rows = [r for r in rows if float(r["dist"]) < 100000 and star_name(r)]
    rows.sort(key=lambda r: float(r["dist"]))

    candidates, seen = [], set()
    for row in rows:
        name = star_name(row)
        if name.lower() not in seen:
            seen.add(name.lower())
            candidates.append((name, row))

    stars, positions = settle(candidates)
    rng = random.Random(SEED)
    pool = traits["pool"][:]
    rng.shuffle(pool)

    galaxy = {}
    for index, ((name, row), (x, y)) in enumerate(zip(stars, positions)):
        entry = traits["start"] if index == 0 else pool.pop()
        galaxy[name] = {"name": name, "x": x, "y": y, **entry,
                        "description": description(name, row, entry)}
    json.dump(galaxy, sys.stdout, indent=1, ensure_ascii=False)
    print()


def layout(stars):
    reach = max(max(abs(float(r["x"])), abs(float(r["y"]))) for _, r in stars)
    scale = (GRID / 2 - 1) / reach
    taken = []
    for _, row in stars:
        x = round(GRID / 2 + float(row["x"]) * scale)
        y = round(GRID / 2 + float(row["y"]) * scale)
        taken.append(nudge(x, y, taken))
    return taken


def settle(candidates):
    """The nearest stars, minus any left with fewer than MIN_NEIGHBOURS in jump range."""
    skipped = set()
    for _ in range(50):
        stars = [c for i, c in enumerate(candidates) if i not in skipped][:SYSTEMS]
        positions = layout(stars)
        lonely = [
            i for i, (x, y) in enumerate(positions)
            if sum(1 for j, (a, b) in enumerate(positions)
                   if j != i and math.hypot(a - x, b - y) / 4 <= JUMP_RANGE) < MIN_NEIGHBOURS
        ]
        if not lonely:
            return stars, positions
        names = {stars[i][0] for i in lonely}
        skipped.update(i for i, c in enumerate(candidates) if c[0] in names)
    raise SystemExit("galaxy did not settle")


def nudge(x, y, taken):
    for ring in range(0, 20):
        for dx in range(-ring, ring + 1):
            for dy in range(-ring, ring + 1):
                if max(abs(dx), abs(dy)) != ring:
                    continue
                cx, cy = x + dx, y + dy
                if not (0 <= cx <= GRID and 0 <= cy <= GRID):
                    continue
                if all(math.hypot(cx - tx, cy - ty) >= MIN_SEPARATION for tx, ty in taken):
                    return cx, cy
    raise SystemExit("no free cell")


if __name__ == "__main__":
    main(sys.argv[1])
