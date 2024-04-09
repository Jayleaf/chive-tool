# Honkai Star Rail Achievement Scanner

## `⚠️ Disclaimer ⚠️`
This program is *USE AT YOUR OWN RISK.* The way this program operates is it will create a handle to the StarRail.exe process, and search the memory for specific achievement IDs and offset them for results. There is some evidence around that EasyAntiCheat (HSR's Anticheat) will detect open handles, though no evidence of this has been found during testing, including leaving a handle open and reading data for around an hour. If you are concerned about EAC detection, please use (this achievement scanner)['https://github.com/hashblen/HSRAchievementScanner/releases/tag/v1.2'].

## Functionality:
- Creates a handle to the StarRail.exe process
- Searches through a specific portion of the program's allocated memory for achievement IDs
- Offsets the memory address to the achievement by 12 to check the status (03 if complete, 01 if incomplete)
- Additionally offsets the memory address to the achievement by 8 for extra verification that it has found the proper address, there should be a 01 in that spot if the check is successful

## Credits:
- https://stardb.gg/api/achievements for achievement ID data
- https://github.com/visibou/lunarengine for allowing me to connect the dots between memory addresses in development in a non-intrusive way (Lunar Engine is not in any way used in this program, this was only used in the development process)
- https://lonami.dev/blog/woce-2/ for helping me with exact-value scanning