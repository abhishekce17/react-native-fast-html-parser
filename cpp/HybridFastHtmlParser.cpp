#include "HybridFastHtmlParser.hpp"

namespace margelo::nitro::fasthtmlparser {

// ── HybridInlineNode ─────────────────────────────────────────────────────────
std::string HybridInlineNode::getType() {
    return getStringAndFree(get_inline_node_type(m_node));
}
std::string HybridInlineNode::getText() {
    return getStringAndFree(get_inline_node_text(m_node));
}
std::string HybridInlineNode::getUrl() {
    return getStringAndFree(get_inline_node_url(m_node));
}
double HybridInlineNode::getChildCount() {
    return (double)get_inline_node_child_count(m_node);
}
std::shared_ptr<HybridInlineNodeSpec> HybridInlineNode::getChild(double index) {
    const InlineNode* child = get_inline_node_child_by_index(m_node, (size_t)index);
    if (!child) return nullptr;
    return std::make_shared<HybridInlineNode>(m_root, child);
}

// ── HybridDefinitionItem ─────────────────────────────────────────────────────
double HybridDefinitionItem::getTermCount() {
    return (double)get_def_item_term_count(m_item);
}
std::shared_ptr<HybridInlineNodeSpec> HybridDefinitionItem::getTerm(double index) {
    const InlineNode* child = get_def_item_term_by_index(m_item, (size_t)index);
    if (!child) return nullptr;
    return std::make_shared<HybridInlineNode>(m_root, child);
}
double HybridDefinitionItem::getDefCount() {
    return (double)get_def_item_def_count(m_item);
}
std::shared_ptr<HybridInlineNodeSpec> HybridDefinitionItem::getDef(double index) {
    const InlineNode* child = get_def_item_def_by_index(m_item, (size_t)index);
    if (!child) return nullptr;
    return std::make_shared<HybridInlineNode>(m_root, child);
}

// ── HybridTableCell ──────────────────────────────────────────────────────────
double HybridTableCell::getChildCount() {
    return (double)get_table_cell_child_count(m_cell);
}
std::shared_ptr<HybridInlineNodeSpec> HybridTableCell::getChild(double index) {
    const InlineNode* child = get_table_cell_child_by_index(m_cell, (size_t)index);
    if (!child) return nullptr;
    return std::make_shared<HybridInlineNode>(m_root, child);
}

// ── HybridTableRow ───────────────────────────────────────────────────────────
double HybridTableRow::getCellCount() {
    return (double)get_table_row_cell_count(m_row);
}
std::shared_ptr<HybridTableCellSpec> HybridTableRow::getCell(double index) {
    const TableCell* cell = get_table_row_cell_by_index(m_row, (size_t)index);
    if (!cell) return nullptr;
    return std::make_shared<HybridTableCell>(m_root, cell);
}

// ── HybridListItem ───────────────────────────────────────────────────────────
double HybridListItem::getChildCount() {
    return (double)get_list_item_child_count(m_item);
}
std::shared_ptr<HybridInlineNodeSpec> HybridListItem::getChild(double index) {
    const InlineNode* child = get_list_item_child_by_index(m_item, (size_t)index);
    if (!child) return nullptr;
    return std::make_shared<HybridInlineNode>(m_root, child);
}
double HybridListItem::getNestedCount() {
    return (double)get_list_item_nested_count(m_item);
}
std::shared_ptr<HybridContentBlockSpec> HybridListItem::getNested(double index) {
    const ContentBlock* nested = get_list_item_nested_by_index(m_item, (size_t)index);
    if (!nested) return nullptr;
    char* raw_type = get_block_type_ptr(nested);
    std::string t(raw_type);
    free_string_ffi(raw_type);
    return std::make_shared<HybridContentBlock>(m_root, nested, t);
}

// ── HybridContentBlock ───────────────────────────────────────────────────────
std::string HybridContentBlock::getType() {
    return m_type;
}
double HybridContentBlock::getLevel() {
    return (double)get_heading_level_ptr(m_block);
}
std::string HybridContentBlock::getUrl() {
    return getStringAndFree(get_image_url_ptr(m_block));
}
std::string HybridContentBlock::getAlt() {
    return getStringAndFree(get_image_alt_ptr(m_block));
}
std::string HybridContentBlock::getCaption() {
    return getStringAndFree(get_figure_caption_ptr(m_block));
}
std::string HybridContentBlock::getCode() {
    return getStringAndFree(get_codeblock_code_ptr(m_block));
}
std::string HybridContentBlock::getLanguage() {
    return getStringAndFree(get_codeblock_lang_ptr(m_block));
}
std::string HybridContentBlock::getSrc() {
    if (m_type == "Video") return getStringAndFree(get_video_src_ptr(m_block));
    if (m_type == "Audio") return getStringAndFree(get_audio_src_ptr(m_block));
    return getStringAndFree(get_embed_src_ptr(m_block));
}
std::string HybridContentBlock::getPoster() {
    return getStringAndFree(get_video_poster_ptr(m_block));
}
std::string HybridContentBlock::getTitle() {
    return getStringAndFree(get_embed_title_ptr(m_block));
}
double HybridContentBlock::getChildCount() {
    return (double)get_block_child_count_ptr(m_block);
}
std::shared_ptr<HybridInlineNodeSpec> HybridContentBlock::getChild(double index) {
    const InlineNode* child = get_block_child_by_index(m_block, (size_t)index);
    if (!child) return nullptr;
    return std::make_shared<HybridInlineNode>(m_root, child);
}
std::shared_ptr<HybridContentBlockSpec> HybridContentBlock::getQuoteChild(double index) {
    const ContentBlock* child = get_quote_child_by_index(m_block, (size_t)index);
    if (!child) return nullptr;
    char* raw_type = get_block_type_ptr(child);
    std::string t(raw_type);
    free_string_ffi(raw_type);
    return std::make_shared<HybridContentBlock>(m_root, child, t);
}
bool HybridContentBlock::getOrdered() {
    return get_list_ordered(m_block);
}
double HybridContentBlock::getItemCount() {
    return (double)get_list_item_count(m_block);
}
std::shared_ptr<HybridListItemSpec> HybridContentBlock::getItem(double index) {
    const ListItem* item = get_list_item_by_index(m_block, (size_t)index);
    if (!item) return nullptr;
    return std::make_shared<HybridListItem>(m_root, item);
}
double HybridContentBlock::getRowCount() {
    return (double)get_table_row_count(m_block);
}
std::shared_ptr<HybridTableRowSpec> HybridContentBlock::getRow(double index) {
    const TableRow* row = get_table_row_by_index(m_block, (size_t)index);
    if (!row) return nullptr;
    return std::make_shared<HybridTableRow>(m_root, row);
}
std::shared_ptr<HybridDefinitionItemSpec> HybridContentBlock::getDefItem(double index) {
    const DefinitionItem* item = get_def_list_item_by_index(m_block, (size_t)index);
    if (!item) return nullptr;
    return std::make_shared<HybridDefinitionItem>(m_root, item);
}

// ── HybridParsedArticle ──────────────────────────────────────────────────────
double HybridParsedArticle::getLength() {
    return (double)get_block_count(m_article);
}
std::shared_ptr<HybridContentBlockSpec> HybridParsedArticle::getBlock(double index) {
    const ContentBlock* block = get_block_by_index(m_article, (size_t)index);
    if (!block) return nullptr;
    char* raw_type = get_block_type_ptr(block);
    std::string t(raw_type);
    free_string_ffi(raw_type);
    return std::make_shared<HybridContentBlock>(shared_from_this(), block, t);
}

// ── HybridFastHtmlParser ─────────────────────────────────────────────────────
std::shared_ptr<HybridParsedArticleSpec> HybridFastHtmlParser::parse(const std::string& html) {
    return std::make_shared<HybridParsedArticle>(html.c_str());
}

} // namespace margelo::nitro::fasthtmlparser
