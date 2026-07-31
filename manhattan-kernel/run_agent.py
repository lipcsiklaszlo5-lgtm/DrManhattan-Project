import os

print("=== DrManhattan Ágens Indítása ===")

# 1. Projekt struktúra ellenőrzése
print("\n[1/3] Projekt struktúra ellenőrzése...")
files_to_check = [
    "src/policy/mod.rs",
    "src/candidate/local_search.rs",
    "src/lib.rs"
]
for f in files_to_check:
    if os.path.exists(f):
        print(f"  [OK] {f} létezik.")
    else:
        print(f"  [HIBA] {f} nem található!")

# 2. Kulcsfontosságú fájlok tartalmának kiírása a debughoz
print("\n[2/3] Kontextus begyűjtése...")
def print_file_head(path, lines_count=40):
    if os.path.exists(path):
        print(f"\n--- {path} (első {lines_count} sor) ---")
        with open(path, 'r') as f:
            lines = f.readlines()
            for line in lines[:lines_count]:
                print(line.rstrip())
    else:
        print(f"\n--- {path} nem található ---")

print_file_head("src/policy/mod.rs")

# 3. Az E2E teszt könyvtár és alap fájl létrehozása (Scaffold)
print("\n[3/3] E2E teszt váz (scaffold) létrehozása...")
os.makedirs("tests", exist_ok=True)
e2e_path = "tests/e2e_pipeline.rs"

e2e_content = """// Automatikusan generált E2E teszt scaffold
use manhattan_kernel::task::TaskBuilder;
use manhattan_kernel::policy::PolicyEngine;

#[test]
fn test_e2e_pipeline_placeholder() {
    // Ez a teszt még csak egy váz, amit a kapott struktúra alapján pontosítunk
    assert!(true);
}
"""

with open(e2e_path, "w") as f:
    f.write(e2e_content)
print(f"  [OK] Létrehozva: {e2e_path}")

print("\n=== Ágens futása kész. Kérlek, másold vissza a fenti outputot! ===")
