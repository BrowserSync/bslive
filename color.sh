
#!/bin/bash

# A shell script to generate and test ANSI escape code variants

# Generate bold, underline, and color combinations
echo -e "Bold: \033[1mThis is bold text\033[0m"
echo -e "Dim: \033[2mThis is dim text\033[0m"
echo -e "Italic: \033[3mThis is italic text (this may not render on all terminals)\033[0m"
echo -e "Underline: \033[4mThis is underlined text\033[0m"
echo -e "Blink: \033[5mThis is blinking text\033[0m (if supported)"
echo -e "Reverse: \033[7mThis is reversed text\033[0m"
echo -e "Hidden: \033[8mThis text is hidden (invisible)\033[0m"

# Foreground color combinations
for fg_color in {30..37} {90..97}; do
  echo -e "\033[${fg_color}mForeground Color $fg_color\033[0m"
done

# Background color combinations
for bg_color in {40..47} {100..107}; do
  echo -e "\033[${bg_color}mBackground Color $bg_color\033[0m"
done

# Combined foreground and background
for fg_color in {30..37} {90..97}; do
  for bg_color in {40..47} {100..107}; do
    echo -e "\033[${fg_color};${bg_color}mForeground $fg_color on Background $bg_color\033[0m"
  done
done

# 256 color mode (if supported)
echo -e "\n256 Color Palette:"
for color in {0..255}; do
  printf "\033[38;5;${color}m%4d\033[0m " $color
  # Add a newline every 16 colors
  if (( (color + 1) % 16 == 0 )); then
    echo
  fi
done

# Test true color mode (24-bit color, if the terminal supports it)
echo -e "\n24-bit True Color Gradient (Red to Blue):"
for r in {0..255..51}; do
  for g in {0..255..51}; do
    for b in {0..255..51}; do
      printf "\033[38;2;${r};${g};${b}m█\033[0m"
    done
    echo
  done
done

echo -e "\nANSI escape code tests complete!"