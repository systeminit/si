#!/usr/bin/ruby

# This tool extracts packet dumps from the TRACE log statements
# when the library compiled with -DLCB_DUMP_PACKETS=ON

require 'stringio'
require 'base64'

def pad(dir, *lines)
  lines.flatten.join("\n").gsub(/^/, dir == 'snd' ? '> ' : '< ')
end

def format_bytes(buf, style = :wide)
  out = StringIO.new
  width = style == :wide ? 32 : 16
  full_rows = buf.size / width
  remainder = buf.size % width

  if style == :wide
    out.print("         +-------------------------------------------------------------------------------------------------+\n" \
              "         |  0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f  0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f |\n" \
              "+--------+-------------------------------------------------------------------------------------------------+--------------------------------+")
  else
    out.print("         +-------------------------------------------------+\n" \
              "         |  0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f |\n" \
              "+--------+-------------------------------------------------+----------------+")
  end

  row = 0
  while row < full_rows
    row_start_index = row * width
    # prefix
    out.printf("\n|%08x|", row_start_index)
    row_end_index = row_start_index + width
    # hex
    i = row_start_index
    while i < row_end_index
      out.printf(" %02x", buf[i].ord)
      i += 1
    end
    out.printf(" |")
    # ascii
    i = row_start_index
    while i < row_end_index
      b = buf[i].ord
      i += 1
      if (b <= 0x1f) || (b >= 0x7f)
        out.printf(".")
      else
        out.printf("%c", b)
      end
    end
    out.printf("|")
    row += 1
  end
  if remainder != 0
    row_start_index = full_rows * width
    # prefix
    out.printf("\n|%08x|", row_start_index)
    row_end_index = row_start_index + remainder
    # hex
    i = row_start_index
    while i < row_end_index
      out.printf(" %02x", buf[i].ord)
      i += 1
    end
    i = width - remainder
    while i > 0
      out.printf("   ")
      i -= 1
    end
    out.printf(" |")
    # ascii
    i = row_start_index
    while i < row_end_index
      b = buf[i].ord
      i += 1
      if (b <= 0x1f) || (b >= 0x7f)
        out.printf(".")
      else
        out.printf("%c", b)
      end
    end
    i = width - remainder
    while i > 0
      out.printf(" ")
      i -= 1
    end
    out.printf("|")
  end
  if style == :wide
    out.print("\n+--------+-------------------------------------------------------------------------------------------------+--------------------------------+\n")
  else
    out.print("\n+--------+-------------------------------------------------+----------------+\n")
  end
  out.string
end

ARGF.each_line do |line|
  line.force_encoding(Encoding::BINARY)
  md = line.match(/TRACE.*<([^>]+)>.*\(CTX=(0x[0-9a-z]+),([^,)]+).*pkt,(snd|rcv).*: size=(\d+), (.+)/)
  next unless md
  address = md[1]
  ctx_id = md[2]
  subsys = md[3]
  dir = md[4]
  data = Base64.decode64(md[6])
  puts pad(dir, "#{address} CTX=#{ctx_id},#{subsys} #{dir == 'snd' ? 'sent' : 'received'} #{data.size} bytes")
  puts pad(dir, format_bytes(data))
end
