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

CLASS_NAMES = {
    "O": "blue", "B": "blue-white", "A": "white", "F": "yellow-white",
    "G": "yellow", "K": "orange", "M": "red",
}


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


def description(row):
    spect = row["spect"].strip()
    colour = CLASS_NAMES.get(spect[:1])
    if row["proper"] == "Sol":
        return "The home system: a yellow class G star."
    if not colour:
        return "A star of unknown class."
    return f"A {colour} class {spect[0]} star."


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
                        "description": description(row)}
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
