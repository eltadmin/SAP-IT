net stop TermService /yes
reg delete "HKLM\SYSTEM\CurrentControlSet\Control\Terminal Server\RCM\GracePeriod" /va /f
sleep 5
net start TermService /yes