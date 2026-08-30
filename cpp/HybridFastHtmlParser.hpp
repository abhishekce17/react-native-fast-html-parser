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
#include <memory>
#include <string>
#include <variant>

extern "C" {
    struct ParsedArticle;
    struct ContentBlock;
    struct ListItem;
    struct TableRow;
    struct TableCell;
    struct DefinitionItem;
    struct InlineNode;

    ParsedArticle* parse_html_ffi(const char* html);
    void free_article_ffi(ParsedArticle* article);
    size_t get_block_count(const ParsedArticle* article);
    void free_string_ffi(char* s);

    const ContentBlock* get_block_by_index(const ParsedArticle* article, size_t index);
    char* get_block_type_ptr(const ContentBlock* block);
    uint8_t get_heading_level_ptr(const ContentBlock* block);
    char* get_image_url_ptr(const ContentBlock* block);
    char* get_image_alt_ptr(const ContentBlock* block);
    char* get_codeblock_code_ptr(const ContentBlock* block);
    char* get_codeblock_lang_ptr(const ContentBlock* block);
    char* get_video_src_ptr(const ContentBlock* block);
    char* get_video_poster_ptr(const ContentBlock* block);
    char* get_audio_src_ptr(const ContentBlock* block);
    char* get_embed_src_ptr(const ContentBlock* block);
    char* get_embed_title_ptr(const ContentBlock* block);
    char* get_figure_caption_ptr(const ContentBlock* block);
    size_t get_block_child_count_ptr(const ContentBlock* block);
    const InlineNode* get_block_child_by_index(const ContentBlock* block, size_t index);
    size_t get_quote_child_count(const ContentBlock* block);
    const ContentBlock* get_quote_child_by_index(const ContentBlock* block, size_t index);
    bool get_list_ordered(const ContentBlock* block);
    size_t get_list_item_count(const ContentBlock* block);
    const ListItem* get_list_item_by_index(const ContentBlock* block, size_t index);
    size_t get_table_row_count(const ContentBlock* block);
    const TableRow* get_table_row_by_index(const ContentBlock* block, size_t index);
    size_t get_def_list_item_count(const ContentBlock* block);
    const DefinitionItem* get_def_list_item_by_index(const ContentBlock* block, size_t index);

    size_t get_list_item_child_count(const ListItem* item);
    const InlineNode* get_list_item_child_by_index(const ListItem* item, size_t index);
    size_t get_list_item_nested_count(const ListItem* item);
    const ContentBlock* get_list_item_nested_by_index(const ListItem* item, size_t index);

    size_t get_table_row_cell_count(const TableRow* row);
    const TableCell* get_table_row_cell_by_index(const TableRow* row, size_t index);
    size_t get_table_cell_child_count(const TableCell* cell);
    const InlineNode* get_table_cell_child_by_index(const TableCell* cell, size_t index);

    size_t get_def_item_term_count(const DefinitionItem* item);
    const InlineNode* get_def_item_term_by_index(const DefinitionItem* item, size_t index);
    size_t get_def_item_def_count(const DefinitionItem* item);
    const InlineNode* get_def_item_def_by_index(const DefinitionItem* item, size_t index);

    char* get_inline_node_type(const InlineNode* node);
    char* get_inline_node_text(const InlineNode* node);
    char* get_inline_node_url(const InlineNode* node);
    size_t get_inline_node_child_count(const InlineNode* node);
    const InlineNode* get_inline_node_child_by_index(const InlineNode* node, size_t index);
}

namespace margelo::nitro::fasthtmlparser {

using namespace margelo::nitro;

inline std::string getStringAndFree(char* raw_str) {
    if (!raw_str) return "";
    std::string s(raw_str);
    free_string_ffi(raw_str);
    return s;
}

class HybridParsedArticle;

// ── HybridInlineNode ─────────────────────────────────────────────────────────
class HybridInlineNode : public HybridInlineNodeSpec {
private:
    std::shared_ptr<HybridParsedArticle> m_root;
    const InlineNode* m_node;
public:
    HybridInlineNode(std::shared_ptr<HybridParsedArticle> root, const InlineNode* node)
        : HybridObject("InlineNode"), HybridInlineNodeSpec(), m_root(root), m_node(node) {}

    std::string getType() override;
    std::string getText() override;
    std::string getUrl() override;
    double getChildCount() override;
    std::variant<std::shared_ptr<HybridInlineNodeSpec>, NullType> getChild(double index) override;
};

// ── HybridDefinitionItem ─────────────────────────────────────────────────────
class HybridDefinitionItem : public HybridDefinitionItemSpec {
private:
    std::shared_ptr<HybridParsedArticle> m_root;
    const DefinitionItem* m_item;
public:
    HybridDefinitionItem(std::shared_ptr<HybridParsedArticle> root, const DefinitionItem* item)
        : HybridObject("DefinitionItem"), HybridDefinitionItemSpec(), m_root(root), m_item(item) {}

    double getTermCount() override;
    std::variant<std::shared_ptr<HybridInlineNodeSpec>, NullType> getTerm(double index) override;
    double getDefCount() override;
    std::variant<std::shared_ptr<HybridInlineNodeSpec>, NullType> getDef(double index) override;
};

// ── HybridTableCell ──────────────────────────────────────────────────────────
class HybridTableCell : public HybridTableCellSpec {
private:
    std::shared_ptr<HybridParsedArticle> m_root;
    const TableCell* m_cell;
public:
    HybridTableCell(std::shared_ptr<HybridParsedArticle> root, const TableCell* cell)
        : HybridObject("TableCell"), HybridTableCellSpec(), m_root(root), m_cell(cell) {}

    double getChildCount() override;
    std::variant<std::shared_ptr<HybridInlineNodeSpec>, NullType> getChild(double index) override;
};

// ── HybridTableRow ───────────────────────────────────────────────────────────
class HybridTableRow : public HybridTableRowSpec {
private:
    std::shared_ptr<HybridParsedArticle> m_root;
    const TableRow* m_row;
public:
    HybridTableRow(std::shared_ptr<HybridParsedArticle> root, const TableRow* row)
        : HybridObject("TableRow"), HybridTableRowSpec(), m_root(root), m_row(row) {}

    double getCellCount() override;
    std::variant<std::shared_ptr<HybridTableCellSpec>, NullType> getCell(double index) override;
};

// ── HybridListItem ───────────────────────────────────────────────────────────
class HybridListItem : public HybridListItemSpec {
private:
    std::shared_ptr<HybridParsedArticle> m_root;
    const ListItem* m_item;
public:
    HybridListItem(std::shared_ptr<HybridParsedArticle> root, const ListItem* item)
        : HybridObject("ListItem"), HybridListItemSpec(), m_root(root), m_item(item) {}

    double getChildCount() override;
    std::variant<std::shared_ptr<HybridInlineNodeSpec>, NullType> getChild(double index) override;
    double getNestedCount() override;
    std::variant<std::shared_ptr<HybridContentBlockSpec>, NullType> getNested(double index) override;
};

// ── HybridContentBlock ───────────────────────────────────────────────────────
class HybridContentBlock : public HybridContentBlockSpec {
private:
    std::shared_ptr<HybridParsedArticle> m_root;
    const ContentBlock* m_block;
    std::string m_type;
public:
    HybridContentBlock(std::shared_ptr<HybridParsedArticle> root, const ContentBlock* block, std::string type)
        : HybridObject("ContentBlock"), HybridContentBlockSpec(), m_root(root), m_block(block), m_type(type) {}

    std::string getType() override;
    double getLevel() override;
    std::string getUrl() override;
    std::string getAlt() override;
    std::string getCaption() override;
    std::string getCode() override;
    std::string getLanguage() override;
    std::string getSrc() override;
    std::string getPoster() override;
    std::string getTitle() override;

    double getChildCount() override;
    std::variant<std::shared_ptr<HybridInlineNodeSpec>, NullType> getChild(double index) override;
    std::variant<std::shared_ptr<HybridContentBlockSpec>, NullType> getQuoteChild(double index) override;

    bool getOrdered() override;
    double getItemCount() override;
    std::variant<std::shared_ptr<HybridListItemSpec>, NullType> getItem(double index) override;

    double getRowCount() override;
    std::variant<std::shared_ptr<HybridTableRowSpec>, NullType> getRow(double index) override;

    std::variant<std::shared_ptr<HybridDefinitionItemSpec>, NullType> getDefItem(double index) override;
};

// ── HybridParsedArticle ──────────────────────────────────────────────────────
// NOTE: do NOT add enable_shared_from_this here — HybridObject already inherits it.
class HybridParsedArticle : public HybridParsedArticleSpec {
private:
    ParsedArticle* m_article;
public:
    HybridParsedArticle(const char* html) : HybridObject("ParsedArticle"), HybridParsedArticleSpec() {
        m_article = parse_html_ffi(html);
    }
    ~HybridParsedArticle() {
        if (m_article) free_article_ffi(m_article);
    }
    const ParsedArticle* getArticle() const { return m_article; }

    double getLength() override;
    std::variant<std::shared_ptr<HybridContentBlockSpec>, NullType> getBlock(double index) override;
};

// ── HybridFastHtmlParser ─────────────────────────────────────────────────────
class HybridFastHtmlParser : public HybridFastHtmlParserSpec {
public:
    HybridFastHtmlParser() : HybridObject("FastHtmlParser"), HybridFastHtmlParserSpec() {}

    std::variant<std::shared_ptr<HybridParsedArticleSpec>, NullType> parse(const std::string& html) override;
};

} // namespace margelo::nitro::fasthtmlparser
