# Moments to review

Finds moments in a video where a person may be in distress or in conflict, for a person to look at: screaming or shouting, crying, struggling to breathe, the sounds of a blow, words of distress or about harming oneself, and aggressive words. A Loomas plugin written in Rust. It uses fixed rules on what other steps found. It has no model, needs no network and downloads nothing.

**What it is not.** It does not decide that a crisis, a panic attack or an attack happened, and it does not judge anyone. It never writes those words: it makes a short list of "moments to review" and says what was heard (`screaming`, `crying`, `words of distress`...). It will flag moments that are fine (films, music, sport, games, a loud laugh) and miss some that are not, and it cannot see anything: a fight in complete silence is invisible to it. It is not a medical device or a safety system. Do not use it to decide anything about a person.

**What it does not look for:** alarms, sirens, gunshots, explosions and other sounds of danger that are not a person. They are clips it must leave alone.

## Privacy

It reads the words people said and the sounds heard, but only as lists that earlier steps made. It writes nothing to disk and uses no network. The recordings can hold people in bad moments: keep them, and what this plugin says about them, to the people who need them.
