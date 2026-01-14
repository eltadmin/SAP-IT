[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

# Function to safely get Last Boot Up Time using multiple methods
function Get-LastBootUpTime {
    try {
        if (Get-Command Get-Uptime -ErrorAction SilentlyContinue) {
            $uptime = Get-Uptime
            $lastBoot = (Get-Date) - $uptime
            return $lastBoot.ToString("yyyy-MM-dd HH:mm:ss")
        } else {
            $event = Get-EventLog -LogName System -Source "Microsoft-Windows-Kernel-General" -InstanceId 12 -Newest 1
            if ($event) {
                return $event.TimeGenerated.ToString("yyyy-MM-dd HH:mm:ss")
            } else {
                Write-Warning "Unable to retrieve LastBootUpTime from event logs."
                return "N/A"
            }
        }
    } catch {
        Write-Warning "Failed to retrieve LastBootUpTime. Using 'N/A'. Error Details: $_"
        return "N/A"
    }
}

# Get the current computer name
$ComputerName = $env:COMPUTERNAME

# Get computer system product information
try {
    $ComputerInfo = Get-CimInstance -ClassName Win32_ComputerSystemProduct
    $Vendor = $ComputerInfo.Vendor
    $Model = $ComputerInfo.Name
    $SerialNumber = $ComputerInfo.IdentifyingNumber
} catch {
    Write-Warning "Failed to retrieve computer system product information. Error Details: $_"
    $Vendor = "Unknown"
    $Model = "Unknown"
    $SerialNumber = "Unknown"
}

# Get other computer details
$OSInfo = Get-CimInstance -ClassName Win32_OperatingSystem
$OSVersion = "$($OSInfo.Caption) $($OSInfo.Version)"
$LastBootUpTime = Get-LastBootUpTime

# Prepare the Snipe-IT API details
$SnipeITBaseURL = "https://inventory.eltrade.com/api/v1"
$SnipeITToken = "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJhdWQiOiIzIiwianRpIjoiNWU4NDQzNTk5MjMzY2ZlMzc5NDAxYzFhMTk2NDBkZDIwYzkxNDM1YTMwYjI2MDk1NjA2NzY3ZGJlNmFiOTRjZjBiYTRkZDk2ZGVkZjJlNjciLCJpYXQiOjE3MjY0NzQ2NjguNTQ3NzE0LCJuYmYiOjE3MjY0NzQ2NjguNTQ3NzE3MSwiZXhwIjoyMTk5NzczODY4LjU0NTM3NDksInN1YiI6IjEiLCJzY29wZXMiOltdfQ.aEXWUEgi4a4K4wx502xLqLWjOREj7DoDVGFEh5UTT1pRXmU14XjqGz5OdGzHFoBw6zsMWfv5-09Dwke8FZGaVqI7xvCSCUqz6DyaPg41WdHeumnI3fr9n-CyMjePouDch37caCD8sxNE2gJ7Ay4fL3Sy05sGMErBmLUx8PvXjIrX9pzq_m7dGxXdgz0r86_Io75lNqABY4TY0EWF9C-59mwiY5WdOQxN1Y0vBO6Gjhqo5ODt3lbTWGDwp1jzGhtYrYAgU4IOuZ6uxu2m270aNxxhkeXllVHSO24kP-lT3Yv2ixgCkaqOJ8NeLtDD79Sx6WWDH6vcTu9fDBF5UxbgPiFga5RSWS3N3O5Th3qF5gnk6T5Dv5whzt9HOyP9lPHERjapsp7zXquTLNzz6yOgAMeQcxRGiNoujc6JKi2K-QK01i97qVqnfrj41hrDML0hKfpJLY_2uyFM6rpvPH1xGWoqaVTiS4uf14W7bjr6FymGEWnIRnOjfa3oRACj4ihjix0ymTJrG3yaZVmvLdbSspWoOhHPBe965qIeJONQxRJu8toCgstOogokMYMrvANPHDdLF4sZNAu22QeEMvCDuNOEfACE0IFTqvTfK0BWdnmVPkIuuMOwD5AF1VymGt0DOClKeryNHPjy2O6-EgF93OL_ih_t6-CK617NIAyO8VU"  # Replace with your actual API token
$ModelID = "1"  # Replace with the ID of the asset model in Snipe-IT
$CompanyID = "1"  # Replace with your company ID in Snipe-IT
$StatusID = "1"  # Replace with the status ID (e.g., Available, Assigned)
$LocationID = "1"  # Replace with the location ID in Snipe-IT

# Function to check if asset exists in SnipeIT by serial number
function Check-SnipeITAsset {
    param (
        [string]$SerialNumber
    )
    $headers = @{
        "Authorization" = "Bearer $SnipeITToken"
        "Accept" = "application/json"
    }
    try {
        $response = Invoke-RestMethod -Uri "$SnipeITBaseURL/hardware?serial=$SerialNumber" -Method Get -Headers $headers -ErrorAction Stop
        return ($response.total -gt 0)
    } catch {
        Write-Warning "Failed to check asset in SnipeIT. Error Details: $_"
        return $false
    }
}

# Function to add asset to SnipeIT
function Add-SnipeITAsset {
    param (
        [string]$AssetTag,
        [string]$SerialNumber,
        [string]$Name,
        [string]$Notes
    )
    $headers = @{
        "Authorization" = "Bearer $SnipeITToken"
        "Accept" = "application/json"
        "Content-Type" = "application/json"
    }
    $body = @{
        "asset_tag" = $AssetTag
        "name" = $Name
        "serial" = $SerialNumber
        "model_id" = $ModelID
        "company_id" = $CompanyID
        "status_id" = $StatusID
        "location_id" = $LocationID
        "notes" = $Notes
    } | ConvertTo-Json

    try {
        $response = Invoke-RestMethod -Uri "$SnipeITBaseURL/hardware" -Method Post -Headers $headers -Body $body -ErrorAction Stop
        if ($response.id) {
            Write-Host "Asset added to SnipeIT successfully. Asset ID: $($response.id)"
        } else {
            Write-Warning "Failed to add asset to SnipeIT. Response: $($response | ConvertTo-Json)"
        }
    } catch {
        Write-Warning "Failed to add asset to SnipeIT. Error Details: $_"
    }
}

# Check and add asset to SnipeIT
if ($SerialNumber -ne "Unknown" -and $SerialNumber -ne "N/A") {
    if (-not (Check-SnipeITAsset -SerialNumber $SerialNumber)) {
        # Use the computer name as the asset tag
        $AssetTag = $ComputerName
        $Name = "$Vendor $Model"
        $Notes = "Added by PowerShell script."
        Add-SnipeITAsset -AssetTag $AssetTag -SerialNumber $SerialNumber -Name $Name -Notes $Notes
    } else {
        Write-Host "Asset with Serial Number $SerialNumber already exists in SnipeIT."
    }
} else {
    Write-Warning "Serial Number is unknown or N/A. Cannot add to SnipeIT."
}

# SIG # Begin signature block
# MIIV+wYJKoZIhvcNAQcCoIIV7DCCFegCAQExCzAJBgUrDgMCGgUAMGkGCisGAQQB
# gjcCAQSgWzBZMDQGCisGAQQBgjcCAR4wJgIDAQAABBAfzDtgWUsITrck0sYpfvNR
# AgEAAgEAAgEAAgEAAgEAMCEwCQYFKw4DAhoFAAQUpF+Cb9tIJZ/C4p5Qoso+82Ja
# 976gghJZMIIFbzCCBFegAwIBAgIQSPyTtGBVlI02p8mKidaUFjANBgkqhkiG9w0B
# AQwFADB7MQswCQYDVQQGEwJHQjEbMBkGA1UECAwSR3JlYXRlciBNYW5jaGVzdGVy
# MRAwDgYDVQQHDAdTYWxmb3JkMRowGAYDVQQKDBFDb21vZG8gQ0EgTGltaXRlZDEh
# MB8GA1UEAwwYQUFBIENlcnRpZmljYXRlIFNlcnZpY2VzMB4XDTIxMDUyNTAwMDAw
# MFoXDTI4MTIzMTIzNTk1OVowVjELMAkGA1UEBhMCR0IxGDAWBgNVBAoTD1NlY3Rp
# Z28gTGltaXRlZDEtMCsGA1UEAxMkU2VjdGlnbyBQdWJsaWMgQ29kZSBTaWduaW5n
# IFJvb3QgUjQ2MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEAjeeUEiIE
# JHQu/xYjApKKtq42haxH1CORKz7cfeIxoFFvrISR41KKteKW3tCHYySJiv/vEpM7
# fbu2ir29BX8nm2tl06UMabG8STma8W1uquSggyfamg0rUOlLW7O4ZDakfko9qXGr
# YbNzszwLDO/bM1flvjQ345cbXf0fEj2CA3bm+z9m0pQxafptszSswXp43JJQ8mTH
# qi0Eq8Nq6uAvp6fcbtfo/9ohq0C/ue4NnsbZnpnvxt4fqQx2sycgoda6/YDnAdLv
# 64IplXCN/7sVz/7RDzaiLk8ykHRGa0c1E3cFM09jLrgt4b9lpwRrGNhx+swI8m2J
# mRCxrds+LOSqGLDGBwF1Z95t6WNjHjZ/aYm+qkU+blpfj6Fby50whjDoA7NAxg0P
# OM1nqFOI+rgwZfpvx+cdsYN0aT6sxGg7seZnM5q2COCABUhA7vaCZEao9XOwBpXy
# bGWfv1VbHJxXGsd4RnxwqpQbghesh+m2yQ6BHEDWFhcp/FycGCvqRfXvvdVnTyhe
# Be6QTHrnxvTQ/PrNPjJGEyA2igTqt6oHRpwNkzoJZplYXCmjuQymMDg80EY2NXyc
# uu7D1fkKdvp+BRtAypI16dV60bV/AK6pkKrFfwGcELEW/MxuGNxvYv6mUKe4e7id
# FT/+IAx1yCJaE5UZkADpGtXChvHjjuxf9OUCAwEAAaOCARIwggEOMB8GA1UdIwQY
# MBaAFKARCiM+lvEH7OKvKe+CpX/QMKS0MB0GA1UdDgQWBBQy65Ka/zWWSC8oQEJw
# IDaRXBeF5jAOBgNVHQ8BAf8EBAMCAYYwDwYDVR0TAQH/BAUwAwEB/zATBgNVHSUE
# DDAKBggrBgEFBQcDAzAbBgNVHSAEFDASMAYGBFUdIAAwCAYGZ4EMAQQBMEMGA1Ud
# HwQ8MDowOKA2oDSGMmh0dHA6Ly9jcmwuY29tb2RvY2EuY29tL0FBQUNlcnRpZmlj
# YXRlU2VydmljZXMuY3JsMDQGCCsGAQUFBwEBBCgwJjAkBggrBgEFBQcwAYYYaHR0
# cDovL29jc3AuY29tb2RvY2EuY29tMA0GCSqGSIb3DQEBDAUAA4IBAQASv6Hvi3Sa
# mES4aUa1qyQKDKSKZ7g6gb9Fin1SB6iNH04hhTmja14tIIa/ELiueTtTzbT72ES+
# BtlcY2fUQBaHRIZyKtYyFfUSg8L54V0RQGf2QidyxSPiAjgaTCDi2wH3zUZPJqJ8
# ZsBRNraJAlTH/Fj7bADu/pimLpWhDFMpH2/YGaZPnvesCepdgsaLr4CnvYFIUoQx
# 2jLsFeSmTD1sOXPUC4U5IOCFGmjhp0g4qdE2JXfBjRkWxYhMZn0vY86Y6GnfrDyo
# XZ3JHFuu2PMvdM+4fvbXg50RlmKarkUT2n/cR/vfw1Kf5gZV6Z2M8jpiUbzsJA8p
# 1FiAhORFe1rYMIIGHDCCBASgAwIBAgIQM9cIqJFAUxnipbvTObmtbjANBgkqhkiG
# 9w0BAQwFADBWMQswCQYDVQQGEwJHQjEYMBYGA1UEChMPU2VjdGlnbyBMaW1pdGVk
# MS0wKwYDVQQDEyRTZWN0aWdvIFB1YmxpYyBDb2RlIFNpZ25pbmcgUm9vdCBSNDYw
# HhcNMjEwMzIyMDAwMDAwWhcNMzYwMzIxMjM1OTU5WjBXMQswCQYDVQQGEwJHQjEY
# MBYGA1UEChMPU2VjdGlnbyBMaW1pdGVkMS4wLAYDVQQDEyVTZWN0aWdvIFB1Ymxp
# YyBDb2RlIFNpZ25pbmcgQ0EgRVYgUjM2MIIBojANBgkqhkiG9w0BAQEFAAOCAY8A
# MIIBigKCAYEAu9H+HrdCW3j1kKeuLIPxjSHTMIaFe9/TzdkWS6yFxbsBz+KMKBFy
# BHYsgcWrEnpASsUQ6IEUORtfTwf2MDAwfzUl5cBzPUAJlOio+Os5C1XVtgyLHif4
# 3j4iwb/vZe5z7mXdKN27H32bMn+3mVUXqrJJqDwQajrDIbKZqEPXO4KoGWG1Pmpa
# Xbi8nhPQCp71W49pOGjqpR9byiPuC+280B5DQ26wU4zCcypEMW6+j7jGAva7ggQV
# eQxSIOiYJ3Fh7y/k+AL7M1m19MNV59/2CCKuttEJWewBn3OJt0NP1fLZvVZZCd23
# F/bEdIC6h0asBtvbBA3VTrrujAk0GZUb5nATBCXfj7jXhDOMbKYM62i6lU98ROjU
# aY0lecMh8TV3+E+2ElWV0FboGALV7nnIhqFp8RtOlBNqB2Lw0GuZpZdQnhwzoR7u
# YYsFaByO9e4mkIPW/nGFp5ryDRQ+NrUSrXd1esznRjZqkFPLxpRx3gc6IfnWMmfg
# nG5UhqBkoIPLAgMBAAGjggFjMIIBXzAfBgNVHSMEGDAWgBQy65Ka/zWWSC8oQEJw
# IDaRXBeF5jAdBgNVHQ4EFgQUgTKSQSsozUbIxKLGKjkS7EipPxQwDgYDVR0PAQH/
# BAQDAgGGMBIGA1UdEwEB/wQIMAYBAf8CAQAwEwYDVR0lBAwwCgYIKwYBBQUHAwMw
# GgYDVR0gBBMwETAGBgRVHSAAMAcGBWeBDAEDMEsGA1UdHwREMEIwQKA+oDyGOmh0
# dHA6Ly9jcmwuc2VjdGlnby5jb20vU2VjdGlnb1B1YmxpY0NvZGVTaWduaW5nUm9v
# dFI0Ni5jcmwwewYIKwYBBQUHAQEEbzBtMEYGCCsGAQUFBzAChjpodHRwOi8vY3J0
# LnNlY3RpZ28uY29tL1NlY3RpZ29QdWJsaWNDb2RlU2lnbmluZ1Jvb3RSNDYucDdj
# MCMGCCsGAQUFBzABhhdodHRwOi8vb2NzcC5zZWN0aWdvLmNvbTANBgkqhkiG9w0B
# AQwFAAOCAgEAXzas+/n2cloUt/ALHd7Y/ZcB0v0B7pkthuj2t/A5/9aBSlqnQkoK
# LRWd5pT9xWlKstdL8RYSTPa+kGZliy101KsI92oRAwh3fL5p4bDbnySJA9beXKTg
# sta0z+M41bltzCfWzmQR6BBydtP54OksielJ07OXlgYK4fYKyEGakV2B2DZ3mMqA
# QZeo+JE/Y5+qzVRUS4Dq9Rdm05Rx/Z79RzHj6RqGHdO+INI/sVJfspO9jJUJmHKP
# lQH0mEOlSvsUJqqdNr9ysPzcvYQN7O00qF6VKzgWYwV12fYxLhVr4pSyKtJ0NbWY
# mqP++CsvthdLJ2xa5rl2XtqG3atk1mrqgxiIGzGC9YizlCXAIS8IaQLjTLtMKhEw
# 64F5BuFBlSrUIPYLk+R8dgydHSZrX4QB9iqZza/ex/DkGKJOmy8qDGamknUmvtlA
# NRNvrqY3GnrorRxRYwcqVgZs7X4Y9uPsZHOmbQg2i68Pma51axcrwk1qw1FGQVbp
# j8KN/xNxm9rtntOfq+VFphLFFFpSQZejBgAIxeYc6ieCPDvb5kbE7y0ANRPNNn2d
# 5aonCAXMzsA2DksZT9Bjmm2/xSlTMSLbdVB3htDy+GruawYbPoUjK5fIfnqZQQzd
# WH8OqMMSPTo1m+CdLIwXgVREqHodmJ2Wf1lYplRl/1FCC/hH68/45b8wggbCMIIF
# KqADAgECAhB2l/+CPxjT4GH/fZVK9M9mMA0GCSqGSIb3DQEBCwUAMFcxCzAJBgNV
# BAYTAkdCMRgwFgYDVQQKEw9TZWN0aWdvIExpbWl0ZWQxLjAsBgNVBAMTJVNlY3Rp
# Z28gUHVibGljIENvZGUgU2lnbmluZyBDQSBFViBSMzYwHhcNMjIxMjExMDAwMDAw
# WhcNMjUxMjEwMjM1OTU5WjCBqTESMBAGA1UEBRMJODMyMDc2MzAyMRMwEQYLKwYB
# BAGCNzwCAQMTAkJHMR0wGwYDVQQPExRQcml2YXRlIE9yZ2FuaXphdGlvbjELMAkG
# A1UEBhMCQkcxDjAMBgNVBAgMBVNvZmlhMSAwHgYDVQQKDBci0JXQm9Ci0KDQldCZ
# 0JQiINCe0J7QlDEgMB4GA1UEAwwXItCV0JvQotCg0JXQmdCUIiDQntCe0JQwggIi
# MA0GCSqGSIb3DQEBAQUAA4ICDwAwggIKAoICAQDdsx6eBJ7wLWmCkVTXJhLcD6cE
# WhIxpTCD5Qp053e/45Zcc1BtfyNXWEfKVh9vKaMlm6b/4IOPAU9ApJGWc4lbBfbV
# dIHa13aNxlIecJ3wk72YPjvMBI1Mye+GJ2PnGNGWSXNnVNvQxm+I7YqgKhbKlOQU
# ccSmDU0GGE0tkRtb68eoQAxBv3nKOpfp0lvcircE2TfTVNr49Jcl+3XOycMtmc7R
# zJr6AllPouF/4sdsDZwpXyVtyW863yxFHwNp8cIt6LgIXUZqXXgzaBNnNbA6bS/i
# EnY5cOor7v1c72HBSOAtxhU8NgK3oPylWJ8zjs4oDPVhcj9byoIbAOTJJ0q8VvoA
# 4rWnOH9onmr7iJ5ppbGkCijhS+stblzFCo5luQWIeqCrvL4RfR/MrxaOXsgYR0dJ
# S+R9iDGT1+mtBwvhE1ociGwGzdPYf2oVuO9nH5V93dKyNhchIddwxYebwR5v37ZC
# fFH3MxYMtmLz2RKUPAX1YRIyEksTBmd+7kScLq6eUw7H43Vp/pVcOVeWE6zBwU+q
# QeuEDsQuQQ6QiRnBBUXsYHa7RMk69GazUNJd0VJ1w/hOOIfE1utnvTlaBW7GzKA2
# ue5k3Z4FcMSUKtxmqLdX1wgAvHLA25PYfap2JNP9hPvd5yK0x50UGCtibC0u6vU9
# BLT66Bm6KZkR37d4+QIDAQABo4IBtTCCAbEwHwYDVR0jBBgwFoAUgTKSQSsozUbI
# xKLGKjkS7EipPxQwHQYDVR0OBBYEFGrFXvcIheBrvGJGR6psI7jbYMMdMA4GA1Ud
# DwEB/wQEAwIHgDAMBgNVHRMBAf8EAjAAMBMGA1UdJQQMMAoGCCsGAQUFBwMDMEkG
# A1UdIARCMEAwNQYMKwYBBAGyMQECAQYBMCUwIwYIKwYBBQUHAgEWF2h0dHBzOi8v
# c2VjdGlnby5jb20vQ1BTMAcGBWeBDAEDMEsGA1UdHwREMEIwQKA+oDyGOmh0dHA6
# Ly9jcmwuc2VjdGlnby5jb20vU2VjdGlnb1B1YmxpY0NvZGVTaWduaW5nQ0FFVlIz
# Ni5jcmwwewYIKwYBBQUHAQEEbzBtMEYGCCsGAQUFBzAChjpodHRwOi8vY3J0LnNl
# Y3RpZ28uY29tL1NlY3RpZ29QdWJsaWNDb2RlU2lnbmluZ0NBRVZSMzYuY3J0MCMG
# CCsGAQUFBzABhhdodHRwOi8vb2NzcC5zZWN0aWdvLmNvbTAnBgNVHREEIDAeoBwG
# CCsGAQUFBwgDoBAwDgwMQkctODMyMDc2MzAyMA0GCSqGSIb3DQEBCwUAA4IBgQAz
# PYZeAZUa2riPdzTrrbYqYcWPfVMLr4xMzCmFV35zyypQE8WQ+C0fgG5OQA3aNyUe
# RecDo+YZFwUhwo9qe+ErF+wzUOOcn22Hfyqv0CLEcfV3RuS11JHvcaubSBp3z7o2
# 8Mn9RuWPLXl+caQMcotqGvAC1I4aSXb+my4TiqJScIbPL2Npyli2J/VeHTUqOho4
# SKrtWALEFEP+IOCI9oBvu8XxY+VlkrTKRPQP0hRTWzD0QqDkk3ABLcgEXbf3QsfO
# bRduhBDsnWS0wyjx5QAeBDdkK+MoyF6DWwcAmD6g7RP73nif+Ofh8pHZxp3SdoDA
# ryVkhOtYEeO9mNj7oDX1P1LkNUXqf/y7yBsXPph/trjcuMm9DS1melhg3XpelYnG
# Xu1hv5zEYC3LtCB6hs9fhR8n2Nrquatv/7e8kmKlu/vCXndPUpbQLqQpeym4IkLJ
# 5KUiO+VPCsYpJEvZ487x5m03OxqpoZdN9RIxflFBwR4O+wBpPf5bnqGqwrZtSDox
# ggMMMIIDCAIBATBrMFcxCzAJBgNVBAYTAkdCMRgwFgYDVQQKEw9TZWN0aWdvIExp
# bWl0ZWQxLjAsBgNVBAMTJVNlY3RpZ28gUHVibGljIENvZGUgU2lnbmluZyBDQSBF
# ViBSMzYCEHaX/4I/GNPgYf99lUr0z2YwCQYFKw4DAhoFAKB4MBgGCisGAQQBgjcC
# AQwxCjAIoAKAAKECgAAwGQYJKoZIhvcNAQkDMQwGCisGAQQBgjcCAQQwHAYKKwYB
# BAGCNwIBCzEOMAwGCisGAQQBgjcCARUwIwYJKoZIhvcNAQkEMRYEFHfiXWlWHR4l
# SPct6Wae8mPISwEFMA0GCSqGSIb3DQEBAQUABIICALonFlAycqrf1yweBH0iZtRI
# ihY3vIx68kTJ+hI32FvsYAKi5nTZtcra9LQI0yCU6/3SzLogc4VkmlYpqH9rF9vb
# izgR24iLjbyK3nx44eQRu4kq4yEyHLszPZWGKZ1FeLyBzeZXBJ0cjzU7xv1yIFiD
# /3/8DdirSuhwuOmBOxUY8CSBaey69IBz28HbxbZgTCEmppaDLp74p0U1Tlc2EoCP
# FszQMrBaSN7yDkFIXJVqawP2mPeCf+sznzoFV7RUlugohmJ3Kllvxgc5w2TB7jbv
# PBWuG5ldNIUVqJk1xWpBktrLlIx2DLRGCjnyCtMoD4DoMwP+/d57id0DzgXSZVBv
# 67iiHv39+ShenywgSLTA4qvsvrsrRX11ZyKtP6CkE4go3AEsXkzL3TEIVBvA+5GH
# EZ0I0UfgJUL4PnXzOAFG5YvkFVl6XHa1qbJp0zZvsejhxlaxD5kQZf/8+WMU00IZ
# dS7sjLwN6ApDRfitufyfjztzuGIUDTtGQYriQPo7w0j+36m47mVky7r5N2ZtxtR4
# gr3MNH8SF4JWgU9WeAJ8O56CNRfDi1uqYGQfT+ujEzi76jNpCQa4lUekp1x6Osq+
# F2TNU5sZ7vLLJ2MB0fvF8Rq7A1z2IWGYwI1JytCYb75rRGtFXBY/uNrDcL4iqiWB
# Bl0LzEqCJDZNCg3pMwsa
# SIG # End signature block
