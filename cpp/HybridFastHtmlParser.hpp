#pragma once

#include "HybridFastHtmlParserSpec.hpp"
#include "HybridParsedArticleSpec.hpp"
#include "HybridContentBlockSpec.hpp"
#include "HybridListItemSpec.hpp"
#include "HybridInlineNodeSpec.hpp"
#include "HybridTableRowSpec.hpp"
#include "HybridTableCellSpec.hpp"
#include "HybridDefinitionItemSpec.hpp"

#include <NitroModules/Null.hpp>
#include <NitroModules/ArrayBuffer.hpp>
#include <NitroModules/Promise.hpp>
#include <memory>
#include <string>
#include <variant>
#include <vector>

namespace margelo::nitro::fasthtmlparser {

using namespace margelo::nitro;

// ── Forward declarations ──────────────────────────────────────────────────────
class HybridContentBlock;
class HybridParsedArticle;

// ── HybridInlineNode ─────────────────────────────────────────────────────────
class HybridInlineNode : public HybridInlineNodeSpec {
public:
    std::string type_;
    std::string text_;
    std::string url_;
    std::vector<std::shared_ptr<HybridInlineNode>> children_;

    HybridInlineNode() : HybridObject("InlineNode"), HybridInlineNodeSpec() {}
    HybridInlineNode(std::string type, std::string text = "", std::string url = "")
        : HybridObject("InlineNode"), HybridInlineNodeSpec(),
          type_(std::move(type)), text_(std::move(text)), url_(std::move(url)) {}

    std::string getType() override { return type_; }
    std::string getText() override { return text_; }
    std::string getUrl() override { return url_; }
    double getChildCount() override { return static_cast<double>(children_.size()); }
    std::variant<std::shared_ptr<HybridInlineNodeSpec>, NullType> getChild(double index) override {
        size_t idx = static_cast<size_t>(index);
        if (idx < children_.size()) return children_[idx];
        return nullptr;
    }
};

// ── HybridTableCell ──────────────────────────────────────────────────────────
class HybridTableCell : public HybridTableCellSpec {
public:
    std::vector<std::shared_ptr<HybridInlineNode>> children_;

    HybridTableCell() : HybridObject("TableCell"), HybridTableCellSpec() {}

    double getChildCount() override { return static_cast<double>(children_.size()); }
    std::variant<std::shared_ptr<HybridInlineNodeSpec>, NullType> getChild(double index) override {
        size_t idx = static_cast<size_t>(index);
        if (idx < children_.size()) return children_[idx];
        return nullptr;
    }
};

// ── HybridTableRow ───────────────────────────────────────────────────────────
class HybridTableRow : public HybridTableRowSpec {
public:
    std::vector<std::shared_ptr<HybridTableCell>> cells_;

    HybridTableRow() : HybridObject("TableRow"), HybridTableRowSpec() {}

    double getCellCount() override { return static_cast<double>(cells_.size()); }
    std::variant<std::shared_ptr<HybridTableCellSpec>, NullType> getCell(double index) override {
        size_t idx = static_cast<size_t>(index);
        if (idx < cells_.size()) return cells_[idx];
        return nullptr;
    }
};

// ── HybridListItem ───────────────────────────────────────────────────────────
class HybridListItem : public HybridListItemSpec {
public:
    std::vector<std::shared_ptr<HybridInlineNode>> children_;
    std::vector<std::shared_ptr<HybridContentBlock>> nested_;

    HybridListItem() : HybridObject("ListItem"), HybridListItemSpec() {}

    double getChildCount() override { return static_cast<double>(children_.size()); }
    std::variant<std::shared_ptr<HybridInlineNodeSpec>, NullType> getChild(double index) override {
        size_t idx = static_cast<size_t>(index);
        if (idx < children_.size()) return children_[idx];
        return nullptr;
    }
    double getNestedCount() override { return static_cast<double>(nested_.size()); }
    std::variant<std::shared_ptr<HybridContentBlockSpec>, NullType> getNested(double index) override;
};

// ── HybridDefinitionItem ─────────────────────────────────────────────────────
class HybridDefinitionItem : public HybridDefinitionItemSpec {
public:
    std::vector<std::shared_ptr<HybridInlineNode>> terms_;
    std::vector<std::shared_ptr<HybridInlineNode>> defs_;

    HybridDefinitionItem() : HybridObject("DefinitionItem"), HybridDefinitionItemSpec() {}

    double getTermCount() override { return static_cast<double>(terms_.size()); }
    std::variant<std::shared_ptr<HybridInlineNodeSpec>, NullType> getTerm(double index) override {
        size_t idx = static_cast<size_t>(index);
        if (idx < terms_.size()) return terms_[idx];
        return nullptr;
    }
    double getDefCount() override { return static_cast<double>(defs_.size()); }
    std::variant<std::shared_ptr<HybridInlineNodeSpec>, NullType> getDef(double index) override {
        size_t idx = static_cast<size_t>(index);
        if (idx < defs_.size()) return defs_[idx];
        return nullptr;
    }
};

// ── HybridContentBlock ───────────────────────────────────────────────────────
class HybridContentBlock : public HybridContentBlockSpec {
public:
    std::string type_;
    double level_{0};
    std::string url_;
    std::string alt_;
    std::string caption_;
    std::string linkUrl_;
    std::string code_;
    std::string language_;
    std::string src_;
    std::string poster_;
    std::string title_;

    std::vector<std::shared_ptr<HybridInlineNode>> children_;
    std::vector<std::shared_ptr<HybridContentBlock>> quoteChildren_;
    bool ordered_{false};
    std::vector<std::shared_ptr<HybridListItem>> items_;
    std::vector<std::shared_ptr<HybridTableRow>> rows_;
    std::vector<std::shared_ptr<HybridDefinitionItem>> defItems_;

    HybridContentBlock() : HybridObject("ContentBlock"), HybridContentBlockSpec() {}
    explicit HybridContentBlock(std::string type)
        : HybridObject("ContentBlock"), HybridContentBlockSpec(), type_(std::move(type)) {}

    std::string getType() override { return type_; }
    double getLevel() override { return level_; }
    std::string getUrl() override { return url_; }
    std::string getAlt() override { return alt_; }
    std::string getCaption() override { return caption_; }
    std::string getLinkUrl() override { return linkUrl_; }
    std::string getCode() override { return code_; }
    std::string getLanguage() override { return language_; }
    std::string getSrc() override { return src_; }
    std::string getPoster() override { return poster_; }
    std::string getTitle() override { return title_; }

    double getChildCount() override { return static_cast<double>(children_.size()); }
    std::variant<std::shared_ptr<HybridInlineNodeSpec>, NullType> getChild(double index) override {
        size_t idx = static_cast<size_t>(index);
        if (idx < children_.size()) return children_[idx];
        return nullptr;
    }
    double getQuoteChildCount() override { return static_cast<double>(quoteChildren_.size()); }
    std::variant<std::shared_ptr<HybridContentBlockSpec>, NullType> getQuoteChild(double index) override {
        size_t idx = static_cast<size_t>(index);
        if (idx < quoteChildren_.size()) return quoteChildren_[idx];
        return nullptr;
    }

    bool getOrdered() override { return ordered_; }
    double getItemCount() override { return static_cast<double>(items_.size()); }
    std::variant<std::shared_ptr<HybridListItemSpec>, NullType> getItem(double index) override {
        size_t idx = static_cast<size_t>(index);
        if (idx < items_.size()) return items_[idx];
        return nullptr;
    }

    double getRowCount() override { return static_cast<double>(rows_.size()); }
    std::variant<std::shared_ptr<HybridTableRowSpec>, NullType> getRow(double index) override {
        size_t idx = static_cast<size_t>(index);
        if (idx < rows_.size()) return rows_[idx];
        return nullptr;
    }

    double getDefItemCount() override { return static_cast<double>(defItems_.size()); }
    std::variant<std::shared_ptr<HybridDefinitionItemSpec>, NullType> getDefItem(double index) override {
        size_t idx = static_cast<size_t>(index);
        if (idx < defItems_.size()) return defItems_[idx];
        return nullptr;
    }
};

inline std::variant<std::shared_ptr<HybridContentBlockSpec>, NullType> HybridListItem::getNested(double index) {
    size_t idx = static_cast<size_t>(index);
    if (idx < nested_.size()) return nested_[idx];
    return nullptr;
}

// ── HybridParsedArticle ──────────────────────────────────────────────────────
class HybridParsedArticle : public HybridParsedArticleSpec {
public:
    std::vector<std::shared_ptr<HybridContentBlock>> blocks_;

    HybridParsedArticle() : HybridObject("ParsedArticle"), HybridParsedArticleSpec() {}
    explicit HybridParsedArticle(std::vector<std::shared_ptr<HybridContentBlock>> blocks)
        : HybridObject("ParsedArticle"), HybridParsedArticleSpec(), blocks_(std::move(blocks)) {}

    double getLength() override { return static_cast<double>(blocks_.size()); }
    std::variant<std::shared_ptr<HybridContentBlockSpec>, NullType> getBlock(double index) override {
        size_t idx = static_cast<size_t>(index);
        if (idx < blocks_.size()) return blocks_[idx];
        return nullptr;
    }
    std::string toJSON() override;
    std::shared_ptr<ArrayBuffer> toBuffer() override;
};

// ── HybridFastHtmlParser ─────────────────────────────────────────────────────
using ParseResult = std::variant<std::shared_ptr<HybridParsedArticleSpec>, NullType>;

class HybridFastHtmlParser : public HybridFastHtmlParserSpec {
public:
    HybridFastHtmlParser() : HybridObject("FastHtmlParser"), HybridFastHtmlParserSpec() {}

    // Sync fast height estimator — no full parse, called before parse() for Frame 0
    double estimateHeight(const std::string& html, double lineHeight) override;

    // Synchronous HTML parse — returns ParsedArticle directly via JSI
    std::variant<std::shared_ptr<HybridParsedArticleSpec>, NullType> parse(const std::string& html) override;

    // Asynchronous HTML parse — dispatches to background thread and returns Promise
    std::shared_ptr<Promise<std::variant<std::shared_ptr<HybridParsedArticleSpec>, NullType>>> parseAsync(const std::string& html) override;

    // JSON serialization helper
    std::string parseToJSON(const std::string& html) override;

    // Internal parse implementation — shared by sync and async modes
    static std::shared_ptr<HybridParsedArticle> parseInternal(const std::string& html);
};

} // namespace margelo::nitro::fasthtmlparser
