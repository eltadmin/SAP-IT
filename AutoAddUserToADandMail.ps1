# Load necessary assemblies for Windows Forms
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

# Import the Active Directory module 
Import-Module ActiveDirectory

# Import the SSH module (install if not already installed)
if (-not (Get-Module -ListAvailable -Name Posh-SSH)) {
    Install-Module -Name Posh-SSH -Force -AllowClobber
}
Import-Module Posh-SSH

# Function to display the interactive GUI menu
function Show-GUIMenu {
    # Create a new form
    $Form = New-Object System.Windows.Forms.Form
    $Form.Text = "User Management Menu"
    $Form.Size = New-Object System.Drawing.Size(400,250)
    $Form.StartPosition = "CenterScreen"
    $Form.FormBorderStyle = 'FixedDialog'
    $Form.MaximizeBox = $false

    # Add buttons for options
    $ButtonAddUser = New-Object System.Windows.Forms.Button
    $ButtonAddUser.Location = New-Object System.Drawing.Point(50,30)
    $ButtonAddUser.Size = New-Object System.Drawing.Size(300,30)
    $ButtonAddUser.Text = "1. Add New User"
    $ButtonAddUser.Add_Click({ $Form.Tag = '1'; $Form.Close() })
    $Form.Controls.Add($ButtonAddUser)

    $ButtonScanUnusedUsers = New-Object System.Windows.Forms.Button
    $ButtonScanUnusedUsers.Location = New-Object System.Drawing.Point(50,70)
    $ButtonScanUnusedUsers.Size = New-Object System.Drawing.Size(300,30)
    $ButtonScanUnusedUsers.Text = "2. Scan for Old Unused User Accounts (+30 days)"
    $ButtonScanUnusedUsers.Add_Click({ $Form.Tag = '2'; $Form.Close() })
    $Form.Controls.Add($ButtonScanUnusedUsers)

    $ButtonScanUnusedComputers = New-Object System.Windows.Forms.Button
    $ButtonScanUnusedComputers.Location = New-Object System.Drawing.Point(50,110)
    $ButtonScanUnusedComputers.Size = New-Object System.Drawing.Size(300,30)
    $ButtonScanUnusedComputers.Text = "3. Scan for Old Unused Computer Accounts (+30 days)"
    $ButtonScanUnusedComputers.Add_Click({ $Form.Tag = '3'; $Form.Close() })
    $Form.Controls.Add($ButtonScanUnusedComputers)

    $ButtonExit = New-Object System.Windows.Forms.Button
    $ButtonExit.Location = New-Object System.Drawing.Point(50,150)
    $ButtonExit.Size = New-Object System.Drawing.Size(300,30)
    $ButtonExit.Text = "4. Exit"
    $ButtonExit.Add_Click({ $Form.Tag = '4'; $Form.Close() })
    $Form.Controls.Add($ButtonExit)

    $Form.ShowDialog()
    return $Form.Tag
}

# Function to add a new user
function Add-NewUser {
    # Create a new form for user input
    $Form = New-Object System.Windows.Forms.Form
    $Form.Text = "Add New User"
    $Form.Size = New-Object System.Drawing.Size(400, 400)
    $Form.StartPosition = "CenterScreen"
    $Form.FormBorderStyle = 'FixedDialog'
    $Form.MaximizeBox = $false
    $Form.AutoSize = $true
    $Form.AutoSizeMode = 'GrowAndShrink'

    # Set a consistent font
    $Form.Font = New-Object System.Drawing.Font("Segoe UI", 9)

    # Create a TableLayoutPanel to organize the controls
    $TableLayout = New-Object System.Windows.Forms.TableLayoutPanel
    $TableLayout.Dock = 'Fill'
    $TableLayout.ColumnCount = 2
    $TableLayout.RowCount = 8
    $TableLayout.Padding = '10,10,10,10'
    $TableLayout.AutoSize = $true
    $TableLayout.AutoSizeMode = 'GrowAndShrink'
    $TableLayout.ColumnStyles.Add((New-Object System.Windows.Forms.ColumnStyle([System.Windows.Forms.SizeType]::Absolute, 150)))
    $TableLayout.ColumnStyles.Add((New-Object System.Windows.Forms.ColumnStyle([System.Windows.Forms.SizeType]::Percent, 100)))

    $Form.Controls.Add($TableLayout)

    # Labels and input controls
    $Labels = @("First Name:", "Last Name:", "Password:", "Department:", "Select Template User:", "Linux Server Password:", "Root Password:")
    $TextBoxes = @()
    $Controls = @()

    for ($i = 0; $i -lt $Labels.Count; $i++) {
        # Create label
        $Label = New-Object System.Windows.Forms.Label
        $Label.Text = $Labels[$i]
        $Label.TextAlign = 'MiddleLeft'
        $Label.AutoSize = $true

        # Add label to the table
        $TableLayout.Controls.Add($Label, 0, $i)

        if ($Labels[$i] -eq "Department:") {
            # ComboBox for department selection
            $ComboBoxDept = New-Object System.Windows.Forms.ComboBox
            $ComboBoxDept.Dock = 'Fill'
            $ComboBoxDept.DropDownStyle = 'DropDownList'
            $Controls += $ComboBoxDept
            $TableLayout.Controls.Add($ComboBoxDept, 1, $i)

            # Populate departments
            $OUBase = "OU=ELTRADE USERS,DC=ELTRADE,DC=COM"
            $Departments = Get-ADOrganizationalUnit -SearchBase $OUBase -Filter * | Select-Object -ExpandProperty Name
            $ComboBoxDept.Items.AddRange($Departments)

            # Event handler to populate template users when department changes
            $ComboBoxDept.Add_SelectedIndexChanged({
                # Clear existing items
                $ComboBoxUser.Items.Clear()

                # Get selected department
                $SelectedDept = $ComboBoxDept.SelectedItem

                # Define OU path based on department
                $OUPath = "OU=$SelectedDept,$OUBase"

                # Get existing users for template selection
                $ExistingUsers = Get-ADUser -Filter * -SearchBase $OUPath | Select-Object -ExpandProperty SamAccountName
                $ComboBoxUser.Items.AddRange($ExistingUsers)
            })
        } elseif ($Labels[$i] -eq "Select Template User:") {
            # ComboBox for template user selection
            $ComboBoxUser = New-Object System.Windows.Forms.ComboBox
            $ComboBoxUser.Dock = 'Fill'
            $ComboBoxUser.DropDownStyle = 'DropDownList'
            $Controls += $ComboBoxUser
            $TableLayout.Controls.Add($ComboBoxUser, 1, $i)
        } elseif ($Labels[$i] -match "Password") {
            # TextBox for passwords
            $TextBox = New-Object System.Windows.Forms.TextBox
            $TextBox.Dock = 'Fill'
            $TextBox.UseSystemPasswordChar = $true
            $TextBoxes += $TextBox
            $Controls += $TextBox
            $TableLayout.Controls.Add($TextBox, 1, $i)
        } else {
            # TextBox for other inputs
            $TextBox = New-Object System.Windows.Forms.TextBox
            $TextBox.Dock = 'Fill'
            $TextBoxes += $TextBox
            $Controls += $TextBox
            $TableLayout.Controls.Add($TextBox, 1, $i)
        }
    }

    # Create a FlowLayoutPanel for the buttons
    $ButtonPanel = New-Object System.Windows.Forms.FlowLayoutPanel
    $ButtonPanel.FlowDirection = 'RightToLeft'
    $ButtonPanel.Dock = 'Bottom'
    $ButtonPanel.Padding = '0,10,10,10'

    $Form.Controls.Add($ButtonPanel)

    # OK button
    $ButtonOK = New-Object System.Windows.Forms.Button
    $ButtonOK.Text = "OK"
    $ButtonOK.Size = New-Object System.Drawing.Size(100, 30)
    $ButtonOK.Add_Click({
        # Validate inputs
        if ($TextBoxes[0].Text -and $TextBoxes[1].Text -and $TextBoxes[2].Text -and $ComboBoxDept.SelectedItem -and $ComboBoxUser.SelectedItem -and $TextBoxes[3].Text -and $TextBoxes[4].Text) {
            $Form.DialogResult = [System.Windows.Forms.DialogResult]::OK
            $Form.Close()
        } else {
            [System.Windows.Forms.MessageBox]::Show("Please fill in all fields.", "Input Required", [System.Windows.Forms.MessageBoxButtons]::OK, [System.Windows.Forms.MessageBoxIcon]::Warning)
        }
    })
    $ButtonPanel.Controls.Add($ButtonOK)

    # Cancel button
    $ButtonCancel = New-Object System.Windows.Forms.Button
    $ButtonCancel.Text = "Cancel"
    $ButtonCancel.Size = New-Object System.Drawing.Size(100, 30)
    $ButtonCancel.Add_Click({
        $Form.DialogResult = [System.Windows.Forms.DialogResult]::Cancel
        $Form.Close()
    })
    $ButtonPanel.Controls.Add($ButtonCancel)

    # Show the form
    $Result = $Form.ShowDialog()

    if ($Result -ne [System.Windows.Forms.DialogResult]::OK) {
        return
    }

    # Collect user inputs
    $FirstName = $TextBoxes[0].Text
    $LastName = $TextBoxes[1].Text
    $Password = ConvertTo-SecureString $TextBoxes[2].Text -AsPlainText -Force
    $Department = $ComboBoxDept.SelectedItem
    $TemplateUser = $ComboBoxUser.SelectedItem
    $LinuxServerPassword = ConvertTo-SecureString $TextBoxes[3].Text -AsPlainText -Force
    $RootPassword = ConvertTo-SecureString $TextBoxes[4].Text -AsPlainText -Force

    if ([string]::IsNullOrWhiteSpace($FirstName) -or [string]::IsNullOrWhiteSpace($LastName)) {
        [System.Windows.Forms.MessageBox]::Show("First Name and Last Name cannot be empty.","Input Error",[System.Windows.Forms.MessageBoxButtons]::OK,[System.Windows.Forms.MessageBoxIcon]::Warning)
        return
    }

    # Generate usernames
    $Username = ($FirstName.Substring(0,1) + "." + $LastName).ToLower()
    $LinuxUsername = ($FirstName.Substring(0,1) + $LastName).ToLower()
    $Email = "$Username@example.com"

    # Define OU path based on department
    $OUPath = "OU=$Department,OU=ELTRADE USERS,DC=ELTRADE,DC=COM"

    # Check if user already exists
    if (Get-ADUser -Filter "SamAccountName -eq '$Username'") {
        [System.Windows.Forms.MessageBox]::Show("A user with the username '$Username' already exists.","User Exists",[System.Windows.Forms.MessageBoxButtons]::OK,[System.Windows.Forms.MessageBoxIcon]::Warning)
        return
    }

    # Create the new user in AD
    Try {
        New-ADUser -Name "$FirstName $LastName" `
                   -GivenName $FirstName `
                   -Surname $LastName `
                   -DisplayName "$FirstName $LastName" `
                   -SamAccountName $Username `
                   -UserPrincipalName $Email `
                   -Path $OUPath `
                   -AccountPassword $Password `
                   -Enabled $true
    } Catch {
        [System.Windows.Forms.MessageBox]::Show("Failed to create user: $($_.Exception.Message)","Error",[System.Windows.Forms.MessageBoxButtons]::OK,[System.Windows.Forms.MessageBoxIcon]::Error)
        return
    }

    # Copy group memberships from template user
    $Groups = Get-ADPrincipalGroupMembership -Identity $TemplateUser
    foreach ($Group in $Groups) {
        # Check if the group is a security group and not built-in
        if ($Group.GroupCategory -eq 'Security' -and $Group.GroupScope -ne 'BuiltinLocal') {
            # Exclude built-in groups
            if ($Group.Name -ne 'Domain Users' -and $Group.Name -ne 'Domain Admins' -and $Group.Name -ne 'Enterprise Admins') {
                # Add user to the group
                Try {
                    Add-ADGroupMember -Identity $Group -Members $Username -ErrorAction Stop
                } Catch {
                    # Log or handle the error
                }
            }
        }
    }

    # Generate random password for the Linux user
    $LinuxUserPassword = ([char[]](33..126) | Get-Random -Count 12) -join ''

    # Connect to Linux server via SSH to create user and update aliases
    $LinuxServer = "172.20.10.10"
    $SSHPort = 12224
    $LinuxUser = "skartselyanski"
    $LinuxCredential = New-Object System.Management.Automation.PSCredential ($LinuxUser, $LinuxServerPassword)

    # Establish SSH session
    Try {
        $SSHSession = New-SSHSession -ComputerName $LinuxServer -Port $SSHPort -Credential $LinuxCredential -AcceptKey
    } Catch {
        $errorMessage = $_.Exception.Message
        [System.Windows.Forms.MessageBox]::Show("Failed to establish SSH session to ${LinuxServer}: ${errorMessage}","SSH Error",[System.Windows.Forms.MessageBoxButtons]::OK,[System.Windows.Forms.MessageBoxIcon]::Error)
        return
    }

    if ($SSHSession -ne $null) {
        Try {
            # Create a Shell Stream to interact with the server
            $ShellStream = New-SSHShellStream -SSHSession $SSHSession

            # Switch to superuser
            $ShellStream.WriteLine("su -")

            # Read output until password prompt
            $Output = ""
            while ($ShellStream.DataAvailable) {
                $Output += $ShellStream.Read()
            }

            # Wait for the password prompt
            while ($Output -notmatch '[Pp]assword:') {
                Start-Sleep -Milliseconds 200
                if ($ShellStream.DataAvailable) {
                    $Output += $ShellStream.Read()
                }
            }

            # Send SU password
            $PlainSUPassword = [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR($RootPassword))
            $ShellStream.WriteLine($PlainSUPassword)

            # Wait for the root prompt
            Start-Sleep -Seconds 1
            $Output = ""
            while ($ShellStream.DataAvailable) {
                $Output += $ShellStream.Read()
            }

            # Now execute commands as root

            # Create new user on Linux
            $AddUserCmd = "useradd -m -c '$FirstName $LastName' $LinuxUsername"
            $ShellStream.WriteLine($AddUserCmd)
            Start-Sleep -Milliseconds 500
            while ($ShellStream.DataAvailable) {
                $Output += $ShellStream.Read()
            }

            # Set password for the Linux user
            $SetPasswordCmd = "echo '${LinuxUsername}:${LinuxUserPassword}' | chpasswd"
            $ShellStream.WriteLine($SetPasswordCmd)
            Start-Sleep -Milliseconds 500
            while ($ShellStream.DataAvailable) {
                $Output += $ShellStream.Read()
            }

            # Update /etc/aliases
            $AliasEntry = "${Username}: ${LinuxUsername}"
            $AddAliasCmd = "sed -i '/###ELTRADE USERS###/a ${AliasEntry}' /etc/aliases"
            $ShellStream.WriteLine($AddAliasCmd)
            Start-Sleep -Milliseconds 500
            while ($ShellStream.DataAvailable) {
                $Output += $ShellStream.Read()
            }

            # Update mail distribution list 'everybody'
            $UpdateDistListCmd = "sed -i '/^everybody:/ s/$/, ${LinuxUsername}/' /etc/aliases"
            $ShellStream.WriteLine($UpdateDistListCmd)
            Start-Sleep -Milliseconds 500
            while ($ShellStream.DataAvailable) {
                $Output += $ShellStream.Read()
            }

            # Rebuild aliases
            $NewAliasesCmd = "newaliases"
            $ShellStream.WriteLine($NewAliasesCmd)
            Start-Sleep -Milliseconds 500
            while ($ShellStream.DataAvailable) {
                $Output += $ShellStream.Read()
            }

            # Exit root shell
            $ShellStream.WriteLine("exit")
            Start-Sleep -Milliseconds 500
            while ($ShellStream.DataAvailable) {
                $Output += $ShellStream.Read()
            }

            # Close shell stream and SSH session
            $ShellStream.Dispose()
            Remove-SSHSession -SSHSession $SSHSession
        } Catch {
            $errorMessage = $_.Exception.Message
            [System.Windows.Forms.MessageBox]::Show("An error occurred during SSH interaction: ${errorMessage}","SSH Error",[System.Windows.Forms.MessageBoxButtons]::OK,[System.Windows.Forms.MessageBoxIcon]::Error)
            $ShellStream.Dispose()
            Remove-SSHSession -SSHSession $SSHSession
            return
        }
    } else {
        [System.Windows.Forms.MessageBox]::Show("Failed to establish SSH session to ${LinuxServer}.","SSH Error",[System.Windows.Forms.MessageBoxButtons]::OK,[System.Windows.Forms.MessageBoxIcon]::Error)
        return
    }

    [System.Windows.Forms.MessageBox]::Show("User $FirstName $LastName added successfully.`nThe Linux password for user '$LinuxUsername' is: $LinuxUserPassword","Success",[System.Windows.Forms.MessageBoxButtons]::OK,[System.Windows.Forms.MessageBoxIcon]::Information)
}

# Function to scan for unused user accounts
function Scan-UnusedAccounts {
    # Calculate the date 30 days ago
    $Date30DaysAgo = (Get-Date).AddDays(-30)
    
    # Users not logged in for the last 30 days
    $UnusedUsers = Get-ADUser -Filter {Enabled -eq $true -and LastLogonTimeStamp -lt $Date30DaysAgo} -Properties LastLogonTimeStamp

    if ($UnusedUsers.Count -eq 0) {
        [System.Windows.Forms.MessageBox]::Show("No unused user accounts found.","Scan Results",[System.Windows.Forms.MessageBoxButtons]::OK,[System.Windows.Forms.MessageBoxIcon]::Information)
    } else {
        $Result = "Unused User Accounts (Not logged in for 30+ days):`n"
        foreach ($User in $UnusedUsers) {
            $LastLogon = [DateTime]::FromFileTime($User.LastLogonTimeStamp)
            $Result += "$($User.Name) - Last Logon: $LastLogon`n"
        }
        [System.Windows.Forms.MessageBox]::Show($Result,"Scan Results",[System.Windows.Forms.MessageBoxButtons]::OK,[System.Windows.Forms.MessageBoxIcon]::Information)
    }
}

# Function to scan for unused computer accounts
function Scan-UnusedComputers {
    # Calculate the date 30 days ago
    $Date30DaysAgo = (Get-Date).AddDays(-30)
    
    # Computers not logged in for the last 30 days
    $UnusedComputers = Get-ADComputer -Filter {LastLogonTimeStamp -lt $Date30DaysAgo} -Properties LastLogonTimeStamp

    if ($UnusedComputers.Count -eq 0) {
        [System.Windows.Forms.MessageBox]::Show("No unused computer accounts found.","Scan Results",[System.Windows.Forms.MessageBoxButtons]::OK,[System.Windows.Forms.MessageBoxIcon]::Information)
    } else {
        $Result = "Unused Computer Accounts (Not logged in for 30+ days):`n"
        foreach ($Computer in $UnusedComputers) {
            $LastLogon = [DateTime]::FromFileTime($Computer.LastLogonTimeStamp)
            $Result += "$($Computer.Name) - Last Logon: $LastLogon`n"
        }
        [System.Windows.Forms.MessageBox]::Show($Result,"Scan Results",[System.Windows.Forms.MessageBoxButtons]::OK,[System.Windows.Forms.MessageBoxIcon]::Information)
    }
}

# Main interactive GUI menu loop
do {
    $Selection = Show-GUIMenu

    switch ($Selection) {
        '1' {
            Add-NewUser
        }
        '2' {
            Scan-UnusedAccounts
        }
        '3' {
            Scan-UnusedComputers
        }
        '4' {
            exit
        }
        Default {
            # Do nothing
        }
    }
} while ($Selection -ne '4')
