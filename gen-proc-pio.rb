#!/usr/bin/env ruby

# Claude wrote the initial version of this. It seems Ok after double checking.

# Read input from stdin or file
input = ARGF.read

# Process line by line
lines = input.lines

# Buffer to collect definitions in a section
section_buffer = []

def flush_section(buffer)
  return if buffer.empty?

  # Calculate max lengths for alignment
  max_name_len = buffer.map { |item| item[:name].length }.max
  max_type_len = buffer.map { |item| item[:type].length }.max

  # Output aligned definitions
  buffer.each do |item|
    name_padded = item[:name].ljust(max_name_len)
    type_padded = item[:type].ljust(max_type_len)
    puts "const #{name_padded} : #{type_padded} = #{item[:value]};"
  end

  buffer.clear
end

lines.each do |line|
  # Skip header guards and includes
  next if line =~ /^#ifndef|^#define.*_DEFINED|^#endif/

  # Pass through comments and flush buffer
  if line =~ /^\s*\/\//
    flush_section(section_buffer)
    puts line
    next
  end

  # Empty lines flush the buffer
  if line.strip.empty?
    flush_section(section_buffer)
    puts line
    next
  end

  next unless line =~ /^#define/

  # Parse #define lines
  if line =~ /^#define\s+(\w+)\s+_u\(([^)]+)\)/
    name = $1
    value = $2.strip

    # Determine type based on value
    type = if value =~ /^".*"$/ || value == '"-"'
             "&str"
           else
             "u32"
           end

    section_buffer << { name: name, type: type, value: value }
  elsif line =~ /^#define\s+(\w+)\s+"([^"]+)"/
    # String without _u()
    name = $1
    value = "\"#{$2}\""
    section_buffer << { name: name, type: "&str", value: value }
  end
end

# Flush any remaining items
flush_section(section_buffer)
