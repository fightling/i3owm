# I3owm

rust implementation of Open Weather Map add-on for i3status

Example usage in i3config:

```
bar {
status_command i3status | i3owm -p 2 -r -k <key> -c Berlin -f '{icon} {temp_c}°C 💧{humidity}%'
}
```

Output would be like:

```
⛅ 11°C 💧55%
```
