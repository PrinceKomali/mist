# Changelog
sorry if the versioning doesnt really make sense, ive been trying to follow semver but i keep messing up
### 1.3.2
- add font size field in config
- allow user selected config, default to assets/default.mts if none
- create config file if missing

### 1.3.1
- add separate fields for timer and split font in config struct
- use font paths from config in app.rs

### 1.3.0
- add configuration file and cfg file parsing
	* config file holds last opened run, colors for timer, path to font
	* colors dont work yet but they will soon
	* custom config not yet selectable
	* will be selectable along with new run when context menu is implemented
- properly save golds on run end

### 1.2.8
- first crates.io published working version
- had to increment version cause i'm stupid

### 1.2.7
- hopefully patch windows file filtering
- add golds for real

### 1.2.6
- reset to top of splits on timer reset
- add preliminary golds suppord
- add proper error handling to msf file parsing

### 1.2.5
- ask to save after rendering last frame (looks much nicer this way)
- on pb, properly update current and pb times and textures of Splits in memory
- only actually save times to chosen file if user agrees to
- fix zero padding, remove extraneous decimals on split times

### 1.2.4
- require split file input path
- patch issue where all splits would happen instantly if you hold down split key

### 1.2.3
- add tinyfiledialog dependency
- add yes/no save splits dialog for writing to msf file
- save run on run end not on splits scroll like a *fool*

### 1.2.2
- fix highlighting the current split when scrolling
- display the proper time when the run ends
- condense some match patterns

### 1.2.1
- properly calculate diffs
- tweak color values

### 1.2.0
- patch color calculation hopefully for the last time
- render diff textures with '+' when behind
- account for pausing in color calculation
- properly clear old textures on timer reset

### 1.1.3
- add split time diff rendering
	* currently no way to handle horizontal resize
	* dynamic color might still be wrong unfortunately

### 1.1.2
- fix dynamic timer color calculation
	* now properly uses making up time color and losing time color
	* still breaks after a pause, will be fixed in a later patch as pausing isnt horribly common

### 1.1.1
- use instant everywhere instead of SDL timer
	* this reduces the number of u32 -> u128 casts
	* also just feels nicer

### 1.1.0
- massive internal changes to split system
	* now uses a wrapper struct for splits to reduce clutter
	* no longer requires large numbers of lifetime-dodging kludges
	* properly implemented `Split` struct field accessing

### 1.0.0
- Basic speedrun timing functionality
- Start offset support
- Read run from split file (file currently locked to "run.msf" in directory where executable is stored)
- If completed run is a PB, save run data to split file
- Change timer color according to run status (not sure if this all works properly)
- Spacebar to start, split, stop; Enter to pause/unpause; R key to reset timer
- Convert time to 30fps values on stop (non-configurable)
- Doesnt crash when you resize the window vertically (yay!) (horizontal resizes probably still bad)