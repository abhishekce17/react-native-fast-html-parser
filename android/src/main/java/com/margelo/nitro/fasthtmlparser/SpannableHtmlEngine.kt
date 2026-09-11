package com.margelo.nitro.fasthtmlparser

import android.content.Context
import android.content.res.Configuration
import android.graphics.Color
import android.text.Spannable
import android.text.SpannableStringBuilder
import android.text.Spanned
import android.text.TextPaint
import android.text.style.*
import android.view.View
import androidx.core.text.HtmlCompat

object SpannableHtmlEngine {

    fun parseColor(colorStr: String?): Int? {
        if (colorStr.isNullOrBlank()) return null
        val trimmed = colorStr.trim()
        return try {
            if (trimmed.startsWith("#")) {
                Color.parseColor(trimmed)
            } else if (trimmed.startsWith("rgba", ignoreCase = true)) {
                val parts = trimmed.substringAfter("(").substringBefore(")").split(",").map { it.trim() }
                if (parts.size == 4) {
                    val r = parts[0].toInt()
                    val g = parts[1].toInt()
                    val b = parts[2].toInt()
                    val a = (parts[3].toFloat() * 255).toInt().coerceIn(0, 255)
                    Color.argb(a, r, g, b)
                } else null
            } else if (trimmed.startsWith("rgb", ignoreCase = true)) {
                val parts = trimmed.substringAfter("(").substringBefore(")").split(",").map { it.trim() }
                if (parts.size == 3) {
                    val r = parts[0].toInt()
                    val g = parts[1].toInt()
                    val b = parts[2].toInt()
                    Color.rgb(r, g, b)
                } else null
            } else if (trimmed.equals("transparent", ignoreCase = true)) {
                Color.TRANSPARENT
            } else {
                null
            }
        } catch (_: Exception) {
            null
        }
    }

    fun buildSpannable(
        context: Context,
        html: String,
        baseStyle: NativeTextStyle?,
        tagsStyles: Map<String, NativeTextStyle>?,
        themeMode: String? = null,
        onLinkPress: ((url: String) -> Unit)?
    ): CharSequence {
        if (html.isEmpty()) return ""

        val isDark = when (themeMode) {
            "dark" -> true
            "light" -> false
            else -> {
                val nightModeFlags = context.resources.configuration.uiMode and Configuration.UI_MODE_NIGHT_MASK
                nightModeFlags == Configuration.UI_MODE_NIGHT_YES
            }
        }

        val defaultLinkColor = if (isDark) Color.parseColor("#38BDF8") else Color.parseColor("#2563EB")
        val defaultQuoteBorder = if (isDark) Color.parseColor("#3B82F6") else Color.parseColor("#2563EB")

        val rawSpanned = try {
            HtmlCompat.fromHtml(html, HtmlCompat.FROM_HTML_MODE_LEGACY)
        } catch (_: Exception) {
            SpannableStringBuilder(html)
        }

        val builder = SpannableStringBuilder(rawSpanned)

        // 1. Intercept URLSpans and replace with Custom ClickableSpan
        val urlSpans = builder.getSpans(0, builder.length, URLSpan::class.java)
        val linkTagStyle = tagsStyles?.get("a")
        val linkColor = parseColor(linkTagStyle?.color) ?: defaultLinkColor

        for (urlSpan in urlSpans) {
            val start = builder.getSpanStart(urlSpan)
            val end = builder.getSpanEnd(urlSpan)
            val flags = builder.getSpanFlags(urlSpan)
            val url = urlSpan.url

            builder.removeSpan(urlSpan)
            builder.setSpan(
                object : ClickableSpan() {
                    override fun onClick(widget: View) {
                        onLinkPress?.invoke(url)
                    }

                    override fun updateDrawState(ds: TextPaint) {
                        super.updateDrawState(ds)
                        ds.color = linkColor
                        ds.isUnderlineText = true
                    }
                },
                start,
                end,
                flags
            )
        }

        // 2. Enhance Blockquote Spans
        val quoteSpans = builder.getSpans(0, builder.length, QuoteSpan::class.java)
        val bqStyle = tagsStyles?.get("blockquote")
        val quoteColor = parseColor(bqStyle?.color) ?: defaultQuoteBorder
        val bqBg = parseColor(bqStyle?.backgroundColor)

        for (qSpan in quoteSpans) {
            val start = builder.getSpanStart(qSpan)
            val end = builder.getSpanEnd(qSpan)
            val flags = builder.getSpanFlags(qSpan)
            builder.removeSpan(qSpan)
            builder.setSpan(
                QuoteSpan(quoteColor),
                start,
                end,
                flags
            )
            if (bqBg != null) {
                builder.setSpan(
                    BackgroundColorSpan(bqBg),
                    start,
                    end,
                    flags
                )
            }
        }

        // 3. Enhance Inline Code Spans
        val defaultCodeBg = if (isDark) Color.parseColor("#1E293B") else Color.parseColor("#E2E8F0")
        val defaultCodeTextColor = if (isDark) Color.parseColor("#38BDF8") else Color.parseColor("#0F172A")
        val codeTagStyle = tagsStyles?.get("code")
        val codeBg = parseColor(codeTagStyle?.backgroundColor) ?: defaultCodeBg
        val codeColor = parseColor(codeTagStyle?.color) ?: defaultCodeTextColor
        val typefaceSpans = builder.getSpans(0, builder.length, TypefaceSpan::class.java)
        for (tSpan in typefaceSpans) {
            val family = tSpan.family
            if (family == null || family.equals("monospace", ignoreCase = true)) {
                val start = builder.getSpanStart(tSpan)
                val end = builder.getSpanEnd(tSpan)
                val flags = builder.getSpanFlags(tSpan)
                builder.setSpan(BackgroundColorSpan(codeBg), start, end, flags)
                builder.setSpan(ForegroundColorSpan(codeColor), start, end, flags)
            }
        }

        // 4. Enhance Bullet Spans with comfortable spacing
        val bulletSpans = builder.getSpans(0, builder.length, BulletSpan::class.java)
        val density = context.resources.displayMetrics.density
        val bulletGap = (12 * density).toInt()
        for (bSpan in bulletSpans) {
            val start = builder.getSpanStart(bSpan)
            val end = builder.getSpanEnd(bSpan)
            val flags = builder.getSpanFlags(bSpan)
            builder.removeSpan(bSpan)
            builder.setSpan(BulletSpan(bulletGap), start, end, flags)
        }

        // 4. Apply Heading Styles from tagsStyles (h1, h2, h3, h4, h5, h6)
        if (tagsStyles != null) {
            val relativeSpans = builder.getSpans(0, builder.length, RelativeSizeSpan::class.java)
            for (rSpan in relativeSpans) {
                val start = builder.getSpanStart(rSpan)
                val end = builder.getSpanEnd(rSpan)
                val flags = builder.getSpanFlags(rSpan)
                val scale = rSpan.sizeChange

                val tag = when {
                    scale >= 1.45f -> "h1"
                    scale >= 1.25f -> "h2"
                    scale >= 1.10f -> "h3"
                    scale >= 0.95f -> "h4"
                    scale >= 0.80f -> "h5"
                    else -> "h6"
                }

                val tagStyle = tagsStyles[tag] ?: continue
                val tagColor = parseColor(tagStyle.color)
                val tagFontSize = tagStyle.fontSize

                if (tagColor != null) {
                    builder.setSpan(ForegroundColorSpan(tagColor), start, end, flags)
                }
                if (tagFontSize != null && tagFontSize > 0) {
                    builder.removeSpan(rSpan)
                    builder.setSpan(AbsoluteSizeSpan(tagFontSize.toInt(), true), start, end, flags)
                }
            }
        }

        return builder
    }
}
