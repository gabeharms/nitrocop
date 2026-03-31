foo = /(?:a+)/
foo = /(?:a*)/
foo = /(?:a?)/
foo = /(?:a*)/
foo = /https?:/
foo = /<.*>/
foo = /a*b/
src.match?(%r{\A(?:https?:)?//player\.example\.com/embed/})
