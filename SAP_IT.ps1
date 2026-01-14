#SERVER LISTS
$servers = @(
    @{ Name = "Ilmatex"; SSH = "root@192.168.0.98"; RDP = "192.168.0.99"; VPN = "ILMATEX" }
    @{ Name = "Frodexim"; SSH = ""; RDP = "192.168.50.20"; VPN = "FRODEXIM" }
    @{ Name = "Industrial Technic"; SSH = "root@192.168.100.10"; RDP = "192.168.100.20"; VPN =  "Industrial Technik" }
    @{ Name = "BG Nova"; SSH = ""; RDP = "192.168.100.20"; VPN =  "Industrial Technik" }
)

#SELECTING A SERVER LOGIC
      
Write-Host ""
Write-Host "Select a server:" -ForegroundColor Cyan
Write-Host "------------------"

$counter = 1

foreach ($server in $servers) {
    
    Write-Host "$counter) $($server.Name)"
    $counter++
}

Write-Host ""
$choice = Read-Host "Enter number"


if ($choice -notmatch '^\d+$' -or $choice -lt 1 -or $choice -gt $servers.Count) {
    Write-Host "Invalid selection." -ForegroundColor Red
    exit 1
}

#SELECTING CONNECTION TYPE

Write-Host ""
Write-Host "Select connection type:" -ForegroundColor Cyan
Write-Host "------------------"

$concounter = 1

$server = $servers[$choice - 1]
if ($server.SSH -ne "") {
$contypes = @(
    @{ Name = "RDP"; }
    @{ Name = "SSH"; }
    @{ Name = "Both"; }
)

$concounter = 1

foreach ($contype in $contypes) {
    Write-Host "$concounter) $($contype.Name)"
    $concounter++
}
$conchoice = Read-Host "Enter Number"
}



if ($conchoice -notmatch '^\d+$' -or $conchoice -lt 1 -or $conchoice -gt $contypes.Count) {
    Write-Host "Invalid selection." -ForegroundColor Red
    exit 1
}



$server = $servers[$choice - 1]
$cc = $contypes[$conchoice - 1]
echo $cc.Name
echo $server.SSH

#CONNECTION LOGIC

if ($cc.Name -eq "RDP" -or $server.SSH -eq "") {
    rasphone -d $server.VPN
    if (Test-Connection $server.RDP -Quiet) {$rdp = Start-Process mstsc.exe /v:$($server.RDP) -PassThru}
    Wait-Process $rdp.Id
}
elseif ($cc.Name -eq "SSH") {
    rasphone -d $server.VPN
    ($server.SSH -match '(?<![\d.])(?:(?:[1-9]?\d|1\d\d|2[0-4]\d|25[0-5])\.){3}(?:[1-9]?\d|1\d\d|2[0-4]\d|25[0-5])(?![\d.])')
    if (Test-Connection $Matches.0 -Quiet) {ssh $server.SSH}
}
elseif ($cc.Name -eq "Both") {
    rasphone -d $server.VPN
    sleep 15
    if (Test-Connection $server.RDP -Quiet) {$rdp = Start-Process mstsc.exe /v:$($server.RDP) -PassThru}
    $sship = ($server.SSH -match '(?<![\d.])(?:(?:[1-9]?\d|1\d\d|2[0-4]\d|25[0-5])\.){3}(?:[1-9]?\d|1\d\d|2[0-4]\d|25[0-5])(?![\d.])')
    if (Test-Connection $Matches.0 -Quiet) {ssh $server.SSH}
    Wait-Process $rdp.Id
}


rasphone -h $server.VPN
