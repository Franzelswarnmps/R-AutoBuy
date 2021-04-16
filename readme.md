# About
This bot was created to allow for easy automation of common browser tasks.
All configuration of the tool is done inside sites.toml, include all the
browser steps.

The bot was originally made to automate buying a 5900X off Amazon,
but it can probably be used for other things. The rest of this readme
is aimed at the amazon use-case.

# Requirements
* Firefox
* Geckodriver executable
* Rust

# ReCaptcha Requirements
* Python3
* amazoncaptcha

# Setup
* Clone the repo locally
* create a Firefox profile to be used by the tool. Login to amazon ahead of time
with "Remember me" checked on this profile
* On the profile, set a specific marionette port, set browser.cache.check_doc_frequency = 1
* In sites.toml, insert the Firefox profile path, and the marionette port
you chose, change the product url, and comment/uncomment the recaptcha step
depending on whether you installed Python3 and amazoncaptcha
* Place the geckodriver.exe in the root of the repo
* cargo run

# Known Issues
* Entering a frame and clicking a button which causes a page change
will cause an unrecoverable error.
* Screenshots don't work becaused the response from webdriver has a strange size
(not equal to width*height of the browser).

# Todo
* Add a config field for geckodriver executable location

# Configuration
You will probably want to change the product url if you're using this to buy
off amazon. If you're using this tool for some new automation, the supplied
sites.toml should give a good example of the capabilities of the bot.