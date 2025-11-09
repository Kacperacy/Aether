# Skrypt do analizy wyników z plików PGN na Windows

param(
    [Parameter(Mandatory=$true, ValueFromRemainingArguments=$true)]
    [string[]]$PgnFiles
)

function Write-Color {
    param([string]$Text, [string]$Color = "White")
    Write-Host $Text -ForegroundColor $Color
}

foreach ($PgnFile in $PgnFiles) {
    if (-not (Test-Path $PgnFile)) {
        Write-Color "BŁĄD: Plik nie istnieje: $PgnFile" "Red"
        continue
    }
    
    Write-Color "=========================================" "Cyan"
    Write-Color "  Analiza: $PgnFile" "Cyan"
    Write-Color "=========================================" "Cyan"
    
    $Content = Get-Content $PgnFile -Raw
    
    # Liczenie wyników
    $TotalGames = ([regex]::Matches($Content, '^\[Result', [System.Text.RegularExpressions.RegexOptions]::Multiline)).Count
    
    # Aether jako białe
    $WhiteAetherMatches = [regex]::Matches($Content, '\[White "Aether"\][\s\S]*?\[Result "([^"]+)"\]')
    $AetherWhiteWins = ($WhiteAetherMatches | Where-Object { $_.Groups[1].Value -eq "1-0" }).Count
    $AetherWhiteDraws = ($WhiteAetherMatches | Where-Object { $_.Groups[1].Value -eq "1/2-1/2" }).Count
    $AetherWhiteLosses = ($WhiteAetherMatches | Where-Object { $_.Groups[1].Value -eq "0-1" }).Count
    
    # Aether jako czarne
    $BlackAetherMatches = [regex]::Matches($Content, '\[Black "Aether"\][\s\S]*?\[Result "([^"]+)"\]')
    $AetherBlackWins = ($BlackAetherMatches | Where-Object { $_.Groups[1].Value -eq "0-1" }).Count
    $AetherBlackDraws = ($BlackAetherMatches | Where-Object { $_.Groups[1].Value -eq "1/2-1/2" }).Count
    $AetherBlackLosses = ($BlackAetherMatches | Where-Object { $_.Groups[1].Value -eq "1-0" }).Count
    
    # Sumy
    $TotalWins = $AetherWhiteWins + $AetherBlackWins
    $TotalDraws = $AetherWhiteDraws + $AetherBlackDraws
    $TotalLosses = $AetherWhiteLosses + $AetherBlackLosses
    
    Write-Host ""
    Write-Host "Całkowite gry: $TotalGames"
    Write-Host ""
    Write-Color "Wyniki Aether:" "Yellow"
    Write-Host "  Wygrane:   $TotalWins (białe: $AetherWhiteWins, czarne: $AetherBlackWins)"
    Write-Host "  Remisy:    $TotalDraws (białe: $AetherWhiteDraws, czarne: $AetherBlackDraws)"
    Write-Host "  Przegrane: $TotalLosses (białe: $AetherWhiteLosses, czarne: $AetherBlackLosses)"
    Write-Host ""
    
    # Procent punktów
    if ($TotalGames -gt 0) {
        $Points = $TotalWins + ($TotalDraws * 0.5)
        $Percentage = [math]::Round(($Points / $TotalGames * 100), 1)
        Write-Host "Wynik: $Points / $TotalGames ($Percentage%)"
        
        # Oszacowanie ELO
        if ($Percentage -gt 75) {
            Write-Color "Szacunkowe ELO: >1700 (dominacja)" "Green"
        } elseif ($Percentage -gt 60) {
            Write-Color "Szacunkowe ELO: ~1650-1700 (przewaga)" "Green"
        } elseif ($Percentage -gt 45) {
            Write-Color "Szacunkowe ELO: ~1500-1600 (równy)" "Yellow"
        } else {
            Write-Color "Szacunkowe ELO: <1500 (słabszy)" "Red"
        }
    }
    
    Write-Host ""
    
    # Średnia długość gry
    $PlyCountMatches = [regex]::Matches($Content, '\[PlyCount "(\d+)"\]')
    if ($PlyCountMatches.Count -gt 0) {
        $TotalPlies = ($PlyCountMatches | ForEach-Object { [int]$_.Groups[1].Value } | Measure-Object -Sum).Sum
        $AvgMoves = [math]::Round($TotalPlies / ($PlyCountMatches.Count * 2), 1)
        Write-Host "Średnia długość gry: $AvgMoves ruchów"
    }
    
    # Przyczyny końca gry
    Write-Host ""
    Write-Color "Przyczyny zakończenia gry:" "Yellow"
    $MateWins = ([regex]::Matches($Content, 'checkmate.*Aether', [System.Text.RegularExpressions.RegexOptions]::IgnoreCase)).Count
    $TimeoutLosses = ([regex]::Matches($Content, 'time.*forfeit', [System.Text.RegularExpressions.RegexOptions]::IgnoreCase)).Count
    Write-Host "  Maty Aether: $MateWins"
    Write-Host "  Przegrane na czas: $TimeoutLosses"
    
    Write-Host ""
}
