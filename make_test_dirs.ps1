# Génère (ou régénère) les dossiers de test de file_sync.
#
#   test_disk/     joue le rôle du disque externe (.filesync, .syncignore, Projets/)
#   test_machine/  joue le rôle du dossier local de la machine
#
# Usage (depuis le dossier file_sync) :
#   .\make_test_dirs.ps1           -> .filesync AVEC BOM (pour reproduire le bug)
#   .\make_test_dirs.ps1 -NoBom    -> .filesync sans BOM
#
# Puis : cargo run -- test_disk
#
# Si PowerShell refuse d'exécuter le script :
#   powershell -ExecutionPolicy Bypass -File .\make_test_dirs.ps1

param([switch]$NoBom)

$ErrorActionPreference = 'Stop'
$here    = $PSScriptRoot
$disk    = Join-Path $here 'test_disk'
$machine = Join-Path $here 'test_machine'

# Repartir de zéro
foreach ($dir in $disk, $machine) {
    if (Test-Path $dir) { Remove-Item $dir -Recurse -Force }
}

# Date de référence fixe : les écarts entre fichiers sont toujours identiques
$base = Get-Date '2026-01-01 10:00:00'

function New-TestFile([string]$path, [string]$content, [datetime]$modified) {
    New-Item -ItemType Directory -Force (Split-Path $path) | Out-Null
    [System.IO.File]::WriteAllText($path, $content)   # UTF-8 sans BOM
    (Get-Item $path).LastWriteTime = $modified
}

# --- Configuration du "disque" ---------------------------------------------
New-Item -ItemType Directory -Force $disk | Out-Null
$machineName = $env:COMPUTERNAME
$filesync = @"
# Fichier de test genere par make_test_dirs.ps1
$($machineName.ToLower()) -> Projets -> $machine\Projets
AUTRE-PC -> Projets -> C:\NeDoitPasApparaitre
"@
$utf8 = New-Object System.Text.UTF8Encoding (-not $NoBom)   # $true = avec BOM
[System.IO.File]::WriteAllText((Join-Path $disk '.filesync'), $filesync, $utf8)

$syncignore = @"
*.blend1
Thumbs.db
/ignore_racine/
"@
[System.IO.File]::WriteAllText((Join-Path $disk '.syncignore'), $syncignore)

# --- Scénarios -------------------------------------------------------------
# Chemins relatifs à Projets/ ; $null = le fichier n'existe pas de ce côté.
# Les dates sont des décalages en minutes par rapport à $base.
$scenarios = @(
    # fichier                                   disque  machine   pull attendu         push attendu
    @{ f = 'identique.txt';                     d = 0;    m = 0    }  # rien               rien
    @{ f = 'tolerance_30s.txt';                 d = 0.5;  m = 0    }  # rien (< 1 min)     rien
    @{ f = 'disque_plus_recent.txt';            d = 120;  m = 0    }  # copier             sauter + avert.
    @{ f = 'machine_plus_recente.txt';          d = 0;    m = 120  }  # sauter + avert.    copier
    @{ f = 'seulement_disque.txt';              d = 0;    m = $null }  # copier             rien
    @{ f = 'seulement_machine.txt';             d = $null; m = 0   }  # rien               copier
    @{ f = 'sous_dossier/profond/nouveau.txt';  d = 0;    m = $null }  # copier + créer dossiers
    @{ f = 'Avec espaces et accents é.txt';     d = 60;   m = 0    }  # copier             sauter + avert.
    @{ f = 'scene.blend1';                      d = 0;    m = $null }  # ignoré (*.blend1)
    @{ f = 'Thumbs.db';                         d = 0;    m = $null }  # ignoré
    @{ f = 'ignore_racine/fichier.txt';         d = 0;    m = $null }  # ignoré (/ignore_racine/)
    @{ f = 'garde/ignore_racine/fichier.txt';   d = 0;    m = $null }  # PAS ignoré (motif ancré)
)

foreach ($s in $scenarios) {
    if ($null -ne $s.d) {
        New-TestFile (Join-Path $disk "Projets/$($s.f)") "version DISQUE de $($s.f)" $base.AddMinutes($s.d)
    }
    if ($null -ne $s.m) {
        New-TestFile (Join-Path $machine "Projets/$($s.f)") "version MACHINE de $($s.f)" $base.AddMinutes($s.m)
    }
}

$bomText = if ($NoBom) { 'sans BOM' } else { 'AVEC BOM' }
Write-Host "Dossiers de test generes ($bomText) :"
Write-Host "  disque  : $disk"
Write-Host "  machine : $machine"
