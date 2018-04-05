define hook-quit
    set confirm off
end

target extended-remote /dev/cu.usbmodemD6CBB8A1
monitor swdp_scan
att 1
load
compare-sections
continue