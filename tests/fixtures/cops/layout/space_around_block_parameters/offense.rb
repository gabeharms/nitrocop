items.each { | x| puts x }
              ^ Layout/SpaceAroundBlockParameters: Space before first block parameter detected.

items.each { |x | puts x }
               ^ Layout/SpaceAroundBlockParameters: Space after last block parameter detected.

items.each { | x | puts x }
              ^ Layout/SpaceAroundBlockParameters: Space before first block parameter detected.
                ^ Layout/SpaceAroundBlockParameters: Space after last block parameter detected.

items.each { |x|puts x }
               ^ Layout/SpaceAroundBlockParameters: Space after closing `|` missing.

handler = proc {|s|cmd.call s}
                  ^ Layout/SpaceAroundBlockParameters: Space after closing `|` missing.
