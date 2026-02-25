# CLI Reference

## Core commands

```bash
tuitbot init
tuitbot auth
tuitbot test
tuitbot run
tuitbot tick
tuitbot stats
tuitbot settings --show
tuitbot health
```

## Operating mode

Switch between Autopilot (fully autonomous) and Composer (user-driven with AI assist):

```bash
tuitbot settings --set mode=composer
tuitbot settings --set mode=autopilot
```

Or via environment variable:

```bash
TUITBOT_MODE=composer tuitbot run
```

In Composer mode, `tuitbot run` and `tuitbot tick` skip the content, thread, discovery (write), mentions, and target monitoring loops. The posting queue, approval poster, and analytics loops continue running.

## Tick mode options

```bash
tuitbot tick --dry-run
tuitbot tick --loops discovery,content,analytics
tuitbot tick --ignore-schedule
tuitbot tick --output json
```

## Update

```bash
tuitbot update              # check for new release + upgrade config
tuitbot update --check      # check only, don't install
tuitbot update --config-only        # skip binary, upgrade config only
tuitbot update --non-interactive    # skip all prompts
```

## Approval queue

```bash
tuitbot approve --list
tuitbot approve --approve <id>
tuitbot approve --reject <id>
tuitbot approve --approve-all
```

## Output modes

Most read-only commands support:

```bash
--output json
```

Use this for automation, monitoring, and alerting.
