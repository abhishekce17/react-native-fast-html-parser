#include "HybridFastHtmlParser.hpp"

#include <lexbor/html/parser.h>
#include <lexbor/html/interfaces/document.h>
#include <lexbor/dom/interfaces/node.h>
#include <lexbor/dom/interfaces/element.h>
#include <lexbor/tag/const.h>

#include <cstring>
#include <vector>
#include <string>
#include <sstream>
#include <functional>
#include <algorithm>
#include <cstdint>

namespace margelo::nitro::fasthtmlparser {

// ── JSON Serialization Helpers ────────────────────────────────────────────────
static void escapeJsonString(const std::string& input, std::ostringstream& ss) {
    ss << '"';
    for (char c : input) {
        switch (c) {
            case '"': ss << "\\\""; break;
            case '\\': ss << "\\\\"; break;
            case '\b': ss << "\\b"; break;
            case '\f': ss << "\\f"; break;
            case '\n': ss << "\\n"; break;
            case '\r': ss << "\\r"; break;
            case '\t': ss << "\\t"; break;
            default:
                if (static_cast<unsigned char>(c) < 0x20) {
                    char buf[7];
                    snprintf(buf, sizeof(buf), "\\u%04x", static_cast<unsigned char>(c));
                    ss << buf;
                } else {
                    ss << c;
                }
                break;
        }
    }
    ss << '"';
}

static void serializeInlineToJson(const std::shared_ptr<HybridInlineNode>& node, std::ostringstream& ss) {
    if (!node) { ss << "null"; return; }
    ss << "{\"type\":";
    escapeJsonString(node->type_, ss);
    if (!node->text_.empty()) {
        ss << ",\"text\":";
        escapeJsonString(node->text_, ss);
    }
    if (!node->url_.empty()) {
        ss << ",\"url\":";
        escapeJsonString(node->url_, ss);
    }
    if (!node->children_.empty()) {
        ss << ",\"children\":[";
        for (size_t i = 0; i < node->children_.size(); ++i) {
            if (i > 0) ss << ",";
            serializeInlineToJson(node->children_[i], ss);
        }
        ss << "]";
    }
    ss << "}";
}

static void serializeBlockToJson(const std::shared_ptr<HybridContentBlock>& block, std::ostringstream& ss) {
    if (!block) { ss << "null"; return; }
    ss << "{\"type\":";
    escapeJsonString(block->type_, ss);
    if (block->level_ > 0) {
        ss << ",\"level\":" << static_cast<int>(block->level_);
    }
    if (!block->url_.empty()) {
        ss << ",\"url\":";
        escapeJsonString(block->url_, ss);
    }
    if (!block->alt_.empty()) {
        ss << ",\"alt\":";
        escapeJsonString(block->alt_, ss);
    }
    if (!block->caption_.empty()) {
        ss << ",\"caption\":";
        escapeJsonString(block->caption_, ss);
    }
    if (!block->linkUrl_.empty()) {
        ss << ",\"linkUrl\":";
        escapeJsonString(block->linkUrl_, ss);
    }
    if (!block->code_.empty()) {
        ss << ",\"code\":";
        escapeJsonString(block->code_, ss);
    }
    if (!block->language_.empty()) {
        ss << ",\"language\":";
        escapeJsonString(block->language_, ss);
    }
    if (!block->src_.empty()) {
        ss << ",\"src\":";
        escapeJsonString(block->src_, ss);
    }
    if (!block->poster_.empty()) {
        ss << ",\"poster\":";
        escapeJsonString(block->poster_, ss);
    }
    if (!block->title_.empty()) {
        ss << ",\"title\":";
        escapeJsonString(block->title_, ss);
    }
    if (!block->children_.empty()) {
        ss << ",\"children\":[";
        for (size_t i = 0; i < block->children_.size(); ++i) {
            if (i > 0) ss << ",";
            serializeInlineToJson(block->children_[i], ss);
        }
        ss << "]";
    }
    if (!block->quoteChildren_.empty()) {
        ss << ",\"children\":[";
        for (size_t i = 0; i < block->quoteChildren_.size(); ++i) {
            if (i > 0) ss << ",";
            serializeBlockToJson(block->quoteChildren_[i], ss);
        }
        ss << "]";
    }
    if (block->type_ == "List") {
        ss << ",\"ordered\":" << (block->ordered_ ? "true" : "false");
        ss << ",\"items\":[";
        for (size_t i = 0; i < block->items_.size(); ++i) {
            if (i > 0) ss << ",";
            ss << "{\"children\":[";
            for (size_t j = 0; j < block->items_[i]->children_.size(); ++j) {
                if (j > 0) ss << ",";
                serializeInlineToJson(block->items_[i]->children_[j], ss);
            }
            ss << "]";
            if (!block->items_[i]->nested_.empty()) {
                ss << ",\"nestedBlocks\":[";
                for (size_t k = 0; k < block->items_[i]->nested_.size(); ++k) {
                    if (k > 0) ss << ",";
                    serializeBlockToJson(block->items_[i]->nested_[k], ss);
                }
                ss << "]";
            }
            ss << "}";
        }
        ss << "]";
    }
    if (block->type_ == "Table") {
        ss << ",\"rows\":[";
        for (size_t i = 0; i < block->rows_.size(); ++i) {
            if (i > 0) ss << ",";
            ss << "{\"cells\":[";
            for (size_t j = 0; j < block->rows_[i]->cells_.size(); ++j) {
                if (j > 0) ss << ",";
                ss << "{\"children\":[";
                for (size_t k = 0; k < block->rows_[i]->cells_[j]->children_.size(); ++k) {
                    if (k > 0) ss << ",";
                    serializeInlineToJson(block->rows_[i]->cells_[j]->children_[k], ss);
                }
                ss << "]}";
            }
            ss << "]}";
        }
        ss << "]";
    }
    if (block->type_ == "DefinitionList") {
        ss << ",\"defItems\":[";
        for (size_t i = 0; i < block->defItems_.size(); ++i) {
            if (i > 0) ss << ",";
            ss << "{\"terms\":[";
            for (size_t j = 0; j < block->defItems_[i]->terms_.size(); ++j) {
                if (j > 0) ss << ",";
                serializeInlineToJson(block->defItems_[i]->terms_[j], ss);
            }
            ss << "],\"defs\":[";
            for (size_t j = 0; j < block->defItems_[i]->defs_.size(); ++j) {
                if (j > 0) ss << ",";
                serializeInlineToJson(block->defItems_[i]->defs_[j], ss);
            }
            ss << "]}";
        }
        ss << "]";
    }
    ss << "}";
}

std::string HybridParsedArticle::toJSON() {
    std::ostringstream ss;
    ss << "{\"blocks\":[";
    for (size_t i = 0; i < blocks_.size(); ++i) {
        if (i > 0) ss << ",";
        serializeBlockToJson(blocks_[i], ss);
    }
    ss << "]}";
    return ss.str();
}

// ── Binary Serialization Helpers ──────────────────────────────────────────────
static void writeUint8(std::vector<uint8_t>& buf, uint8_t val) {
    buf.push_back(val);
}

static void writeUint32(std::vector<uint8_t>& buf, uint32_t val) {
    buf.push_back(static_cast<uint8_t>(val & 0xFF));
    buf.push_back(static_cast<uint8_t>((val >> 8) & 0xFF));
    buf.push_back(static_cast<uint8_t>((val >> 16) & 0xFF));
    buf.push_back(static_cast<uint8_t>((val >> 24) & 0xFF));
}

static void writeString(std::vector<uint8_t>& buf, const std::string& str) {
    writeUint32(buf, static_cast<uint32_t>(str.size()));
    buf.insert(buf.end(), str.begin(), str.end());
}

static void serializeInlineToBinary(const std::shared_ptr<HybridInlineNode>& node, std::vector<uint8_t>& buf) {
    if (!node) {
        writeString(buf, "");
        writeString(buf, "");
        writeString(buf, "");
        writeUint32(buf, 0);
        return;
    }
    writeString(buf, node->type_);
    writeString(buf, node->text_);
    writeString(buf, node->url_);
    writeUint32(buf, static_cast<uint32_t>(node->children_.size()));
    for (const auto& child : node->children_) {
        serializeInlineToBinary(child, buf);
    }
}

static void serializeBlockToBinary(const std::shared_ptr<HybridContentBlock>& block, std::vector<uint8_t>& buf) {
    if (!block) return;
    writeString(buf, block->type_);
    writeUint32(buf, static_cast<uint32_t>(block->level_));
    writeString(buf, block->url_);
    writeString(buf, block->alt_);
    writeString(buf, block->caption_);
    writeString(buf, block->linkUrl_);
    writeString(buf, block->code_);
    writeString(buf, block->language_);
    writeString(buf, block->src_);
    writeString(buf, block->poster_);
    writeString(buf, block->title_);

    // Children
    writeUint32(buf, static_cast<uint32_t>(block->children_.size()));
    for (const auto& in : block->children_) {
        serializeInlineToBinary(in, buf);
    }

    // Quote children
    writeUint32(buf, static_cast<uint32_t>(block->quoteChildren_.size()));
    for (const auto& qb : block->quoteChildren_) {
        serializeBlockToBinary(qb, buf);
    }

    // List items
    writeUint8(buf, block->ordered_ ? 1 : 0);
    writeUint32(buf, static_cast<uint32_t>(block->items_.size()));
    for (const auto& item : block->items_) {
        writeUint32(buf, static_cast<uint32_t>(item->children_.size()));
        for (const auto& in : item->children_) {
            serializeInlineToBinary(in, buf);
        }
        writeUint32(buf, static_cast<uint32_t>(item->nested_.size()));
        for (const auto& nb : item->nested_) {
            serializeBlockToBinary(nb, buf);
        }
    }

    // Table rows
    writeUint32(buf, static_cast<uint32_t>(block->rows_.size()));
    for (const auto& row : block->rows_) {
        writeUint32(buf, static_cast<uint32_t>(row->cells_.size()));
        for (const auto& cell : row->cells_) {
            writeUint32(buf, static_cast<uint32_t>(cell->children_.size()));
            for (const auto& in : cell->children_) {
                serializeInlineToBinary(in, buf);
            }
        }
    }

    // Definition items
    writeUint32(buf, static_cast<uint32_t>(block->defItems_.size()));
    for (const auto& defItem : block->defItems_) {
        writeUint32(buf, static_cast<uint32_t>(defItem->terms_.size()));
        for (const auto& in : defItem->terms_) {
            serializeInlineToBinary(in, buf);
        }
        writeUint32(buf, static_cast<uint32_t>(defItem->defs_.size()));
        for (const auto& in : defItem->defs_) {
            serializeInlineToBinary(in, buf);
        }
    }
}

std::shared_ptr<ArrayBuffer> HybridParsedArticle::toBuffer() {
    std::vector<uint8_t> buf;
    // Magic: 0xF4, 0x54, 0x4D, 0x4C ("\xF4TML")
    buf.push_back(0xF4);
    buf.push_back(0x54);
    buf.push_back(0x4D);
    buf.push_back(0x4C);
    // Version: 2
    buf.push_back(0x02);
    // Block count
    writeUint32(buf, static_cast<uint32_t>(blocks_.size()));
    for (const auto& block : blocks_) {
        serializeBlockToBinary(block, buf);
    }
    return ArrayBuffer::copy(buf);
}

// ── Lexbor DOM Parsing Helpers ────────────────────────────────────────────────
static std::string getAttribute(lxb_dom_element_t* element, const char* name) {
    if (!element) return "";
    size_t val_len = 0;
    const lxb_char_t* val = lxb_dom_element_get_attribute(
        element,
        reinterpret_cast<const lxb_char_t*>(name),
        strlen(name),
        &val_len
    );
    if (val && val_len > 0) {
        return std::string(reinterpret_cast<const char*>(val), val_len);
    }
    return "";
}

static std::string getNodeTagName(lxb_dom_node_t* node) {
    if (!node) return "";
    size_t len = 0;
    const lxb_char_t* name = lxb_dom_node_name(node, &len);
    if (name && len > 0) {
        return std::string(reinterpret_cast<const char*>(name), len);
    }
    return "";
}

static std::string getNodeText(lxb_dom_node_t* node) {
    if (!node) return "";
    size_t len = 0;
    lxb_char_t* text = lxb_dom_node_text_content(node, &len);
    std::string result = (text && len > 0) ? std::string(reinterpret_cast<const char*>(text), len) : "";
    if (text && node->owner_document) {
        lxb_dom_document_destroy_text(node->owner_document, text);
    }
    return result;
}

static std::shared_ptr<HybridInlineNode> parseInlineNode(lxb_dom_node_t* node) {
    if (!node) return nullptr;

    if (node->type == LXB_DOM_NODE_TYPE_TEXT) {
        size_t len = 0;
        lxb_char_t* text = lxb_dom_node_text_content(node, &len);
        std::string str = (text && len > 0) ? std::string(reinterpret_cast<const char*>(text), len) : "";
        if (text && node->owner_document) {
            lxb_dom_document_destroy_text(node->owner_document, text);
        }
        if (str.empty()) return nullptr;
        return std::make_shared<HybridInlineNode>("Text", str, "");
    }

    if (node->type == LXB_DOM_NODE_TYPE_ELEMENT) {
        lxb_tag_id_t tagId = lxb_dom_node_tag_id(node);
        lxb_dom_element_t* elem = lxb_dom_interface_element(node);

        // Skip non-rendered inline elements
        if (tagId == LXB_TAG_SCRIPT || tagId == LXB_TAG_STYLE) {
            return nullptr;
        }

        std::string type = "Span";
        std::string url = "";

        switch (tagId) {
            case LXB_TAG_B:
            case LXB_TAG_STRONG:
                type = "Bold";
                break;
            case LXB_TAG_I:
            case LXB_TAG_EM:
                type = "Italic";
                break;
            case LXB_TAG_A:
                type = "Link";
                url = getAttribute(elem, "href");
                break;
            case LXB_TAG_CODE:
            case LXB_TAG_KBD:
            case LXB_TAG_SAMP:
                type = "Code";
                break;
            case LXB_TAG_S:
            case LXB_TAG_STRIKE:
            case LXB_TAG_DEL:
                type = "Strikethrough";
                break;
            case LXB_TAG_U:
            case LXB_TAG_INS:
                type = "Underline";
                break;
            case LXB_TAG_SUP:
                type = "Superscript";
                break;
            case LXB_TAG_SUB:
                type = "Subscript";
                break;
            case LXB_TAG_MARK:
                type = "Mark";
                break;
            case LXB_TAG_SMALL:
                type = "Small";
                break;
            case LXB_TAG_BR:
            case LXB_TAG_WBR:
                return std::make_shared<HybridInlineNode>("Break", "", "");
            default:
                type = "Span";
                break;
        }

        auto inlineNode = std::make_shared<HybridInlineNode>(type, "", url);
        lxb_dom_node_t* child = node->first_child;
        while (child) {
            auto childInline = parseInlineNode(child);
            if (childInline) {
                inlineNode->children_.push_back(childInline);
            }
            child = child->next;
        }
        return inlineNode;
    }

    return nullptr;
}

static void collectInlineChildren(lxb_dom_node_t* node, std::vector<std::shared_ptr<HybridInlineNode>>& out) {
    lxb_dom_node_t* child = node->first_child;
    while (child) {
        auto in = parseInlineNode(child);
        if (in) out.push_back(in);
        child = child->next;
    }
}

// Forward declaration of DOM walker
static void walkDomNode(lxb_dom_node_t* node, std::vector<std::shared_ptr<HybridContentBlock>>& blocks);

static void walkDomChildren(lxb_dom_node_t* parent, std::vector<std::shared_ptr<HybridContentBlock>>& blocks) {
    if (!parent) return;
    lxb_dom_node_t* child = parent->first_child;
    while (child) {
        walkDomNode(child, blocks);
        child = child->next;
    }
}

static void walkDomNode(lxb_dom_node_t* node, std::vector<std::shared_ptr<HybridContentBlock>>& blocks) {
    if (!node) return;

    if (node->type == LXB_DOM_NODE_TYPE_TEXT) {
        size_t len = 0;
        lxb_char_t* text = lxb_dom_node_text_content(node, &len);
        std::string str = (text && len > 0) ? std::string(reinterpret_cast<const char*>(text), len) : "";
        if (text && node->owner_document) {
            lxb_dom_document_destroy_text(node->owner_document, text);
        }
        // Trim whitespace
        size_t first = str.find_first_not_of(" \t\n\r");
        if (first != std::string::npos) {
            auto p = std::make_shared<HybridContentBlock>("Paragraph");
            p->children_.push_back(std::make_shared<HybridInlineNode>("Text", str, ""));
            blocks.push_back(p);
        }
        return;
    }

    if (node->type != LXB_DOM_NODE_TYPE_ELEMENT) {
        return;
    }

    lxb_tag_id_t tagId = lxb_dom_node_tag_id(node);
    lxb_dom_element_t* elem = lxb_dom_interface_element(node);

    // Skip script, style, head, meta, link, template
    if (tagId == LXB_TAG_SCRIPT || tagId == LXB_TAG_STYLE || tagId == LXB_TAG_HEAD ||
        tagId == LXB_TAG_TITLE || tagId == LXB_TAG_META || tagId == LXB_TAG_LINK ||
        tagId == LXB_TAG_TEMPLATE) {
        return;
    }

    // Headings: <h1> to <h6>
    if (tagId >= LXB_TAG_H1 && tagId <= LXB_TAG_H6) {
        auto block = std::make_shared<HybridContentBlock>("Heading");
        block->level_ = static_cast<double>(tagId - LXB_TAG_H1 + 1);
        collectInlineChildren(node, block->children_);
        blocks.push_back(block);
        return;
    }

    // Paragraph
    if (tagId == LXB_TAG_P) {
        auto block = std::make_shared<HybridContentBlock>("Paragraph");
        collectInlineChildren(node, block->children_);
        blocks.push_back(block);
        return;
    }

    // Blockquote
    if (tagId == LXB_TAG_BLOCKQUOTE) {
        auto block = std::make_shared<HybridContentBlock>("Quote");
        bool hasBlockChildren = false;
        lxb_dom_node_t* child = node->first_child;
        while (child) {
            if (child->type == LXB_DOM_NODE_TYPE_ELEMENT) {
                lxb_tag_id_t tid = lxb_dom_node_tag_id(child);
                if (tid == LXB_TAG_P || (tid >= LXB_TAG_H1 && tid <= LXB_TAG_H6) ||
                    tid == LXB_TAG_UL || tid == LXB_TAG_OL || tid == LXB_TAG_BLOCKQUOTE ||
                    tid == LXB_TAG_PRE || tid == LXB_TAG_TABLE) {
                    hasBlockChildren = true;
                    break;
                }
            }
            child = child->next;
        }

        if (hasBlockChildren) {
            walkDomChildren(node, block->quoteChildren_);
        } else {
            auto p = std::make_shared<HybridContentBlock>("Paragraph");
            collectInlineChildren(node, p->children_);
            block->quoteChildren_.push_back(p);
            collectInlineChildren(node, block->children_);
        }
        blocks.push_back(block);
        return;
    }

    // Pre / CodeBlock
    if (tagId == LXB_TAG_PRE) {
        auto block = std::make_shared<HybridContentBlock>("CodeBlock");
        lxb_dom_node_t* codeNode = nullptr;
        lxb_dom_node_t* c = node->first_child;
        while (c) {
            if (c->type == LXB_DOM_NODE_TYPE_ELEMENT && lxb_dom_node_tag_id(c) == LXB_TAG_CODE) {
                codeNode = c;
                break;
            }
            c = c->next;
        }
        if (codeNode) {
            lxb_dom_element_t* codeElem = lxb_dom_interface_element(codeNode);
            std::string cls = getAttribute(codeElem, "class");
            if (cls.rfind("language-", 0) == 0) {
                block->language_ = cls.substr(9);
            } else if (!cls.empty()) {
                block->language_ = cls;
            }
            block->code_ = getNodeText(codeNode);
        } else {
            block->code_ = getNodeText(node);
        }
        blocks.push_back(block);
        return;
    }

    // Lists: <ul>, <ol>
    if (tagId == LXB_TAG_UL || tagId == LXB_TAG_OL) {
        auto block = std::make_shared<HybridContentBlock>("List");
        block->ordered_ = (tagId == LXB_TAG_OL);
        lxb_dom_node_t* li = node->first_child;
        while (li) {
            if (li->type == LXB_DOM_NODE_TYPE_ELEMENT && lxb_dom_node_tag_id(li) == LXB_TAG_LI) {
                auto item = std::make_shared<HybridListItem>();
                lxb_dom_node_t* liChild = li->first_child;
                while (liChild) {
                    if (liChild->type == LXB_DOM_NODE_TYPE_ELEMENT) {
                        lxb_tag_id_t liChildTag = lxb_dom_node_tag_id(liChild);
                        if (liChildTag == LXB_TAG_UL || liChildTag == LXB_TAG_OL) {
                            std::vector<std::shared_ptr<HybridContentBlock>> nestedBlocks;
                            walkDomNode(liChild, nestedBlocks);
                            for (auto& nb : nestedBlocks) {
                                item->nested_.push_back(nb);
                            }
                            liChild = liChild->next;
                            continue;
                        }
                    }
                    auto inlineChild = parseInlineNode(liChild);
                    if (inlineChild) {
                        item->children_.push_back(inlineChild);
                    }
                    liChild = liChild->next;
                }
                block->items_.push_back(item);
            }
            li = li->next;
        }
        blocks.push_back(block);
        return;
    }

    // Table
    if (tagId == LXB_TAG_TABLE) {
        auto block = std::make_shared<HybridContentBlock>("Table");
        auto processRow = [&](lxb_dom_node_t* tr) {
            auto row = std::make_shared<HybridTableRow>();
            lxb_dom_node_t* cell = tr->first_child;
            while (cell) {
                if (cell->type == LXB_DOM_NODE_TYPE_ELEMENT) {
                    lxb_tag_id_t cellTag = lxb_dom_node_tag_id(cell);
                    if (cellTag == LXB_TAG_TH || cellTag == LXB_TAG_TD) {
                        auto tableCell = std::make_shared<HybridTableCell>();
                        collectInlineChildren(cell, tableCell->children_);
                        row->cells_.push_back(tableCell);
                    }
                }
                cell = cell->next;
            }
            if (!row->cells_.empty()) {
                block->rows_.push_back(row);
            }
        };

        std::function<void(lxb_dom_node_t*)> walkTable = [&](lxb_dom_node_t* tNode) {
            lxb_dom_node_t* child = tNode->first_child;
            while (child) {
                if (child->type == LXB_DOM_NODE_TYPE_ELEMENT) {
                    lxb_tag_id_t tid = lxb_dom_node_tag_id(child);
                    if (tid == LXB_TAG_TR) {
                        processRow(child);
                    } else if (tid == LXB_TAG_THEAD || tid == LXB_TAG_TBODY || tid == LXB_TAG_TFOOT) {
                        walkTable(child);
                    }
                }
                child = child->next;
            }
        };
        walkTable(node);
        blocks.push_back(block);
        return;
    }

    // Definition list: <dl>
    if (tagId == LXB_TAG_DL) {
        auto block = std::make_shared<HybridContentBlock>("DefinitionList");
        lxb_dom_node_t* dlChild = node->first_child;
        std::shared_ptr<HybridDefinitionItem> currentItem = nullptr;
        while (dlChild) {
            if (dlChild->type == LXB_DOM_NODE_TYPE_ELEMENT) {
                lxb_tag_id_t tid = lxb_dom_node_tag_id(dlChild);
                if (tid == LXB_TAG_DT) {
                    currentItem = std::make_shared<HybridDefinitionItem>();
                    collectInlineChildren(dlChild, currentItem->terms_);
                    block->defItems_.push_back(currentItem);
                } else if (tid == LXB_TAG_DD) {
                    if (!currentItem) {
                        currentItem = std::make_shared<HybridDefinitionItem>();
                        block->defItems_.push_back(currentItem);
                    }
                    collectInlineChildren(dlChild, currentItem->defs_);
                }
            }
            dlChild = dlChild->next;
        }
        blocks.push_back(block);
        return;
    }

    // Image: <img>
    if (tagId == LXB_TAG_IMG) {
        auto block = std::make_shared<HybridContentBlock>("Image");
        block->url_ = getAttribute(elem, "src");
        block->alt_ = getAttribute(elem, "alt");
        block->title_ = getAttribute(elem, "title");
        blocks.push_back(block);
        return;
    }

    // Figure: <figure>
    if (tagId == LXB_TAG_FIGURE) {
        auto block = std::make_shared<HybridContentBlock>("Figure");
        std::function<void(lxb_dom_node_t*)> scanFigure = [&](lxb_dom_node_t* fNode) {
            lxb_dom_node_t* c = fNode->first_child;
            while (c) {
                if (c->type == LXB_DOM_NODE_TYPE_ELEMENT) {
                    lxb_tag_id_t tid = lxb_dom_node_tag_id(c);
                    if (tid == LXB_TAG_IMG) {
                        lxb_dom_element_t* imgElem = lxb_dom_interface_element(c);
                        block->url_ = getAttribute(imgElem, "src");
                        block->alt_ = getAttribute(imgElem, "alt");
                    } else if (tid == LXB_TAG_FIGCAPTION) {
                        block->caption_ = getNodeText(c);
                    } else {
                        scanFigure(c);
                    }
                }
                c = c->next;
            }
        };
        scanFigure(node);
        blocks.push_back(block);
        return;
    }

    // Video: <video>
    if (tagId == LXB_TAG_VIDEO) {
        auto block = std::make_shared<HybridContentBlock>("Video");
        block->src_ = getAttribute(elem, "src");
        block->poster_ = getAttribute(elem, "poster");
        if (block->src_.empty()) {
            lxb_dom_node_t* c = node->first_child;
            while (c) {
                if (c->type == LXB_DOM_NODE_TYPE_ELEMENT && lxb_dom_node_tag_id(c) == LXB_TAG_SOURCE) {
                    block->src_ = getAttribute(lxb_dom_interface_element(c), "src");
                    if (!block->src_.empty()) break;
                }
                c = c->next;
            }
        }
        blocks.push_back(block);
        return;
    }

    // Audio: <audio>
    if (tagId == LXB_TAG_AUDIO) {
        auto block = std::make_shared<HybridContentBlock>("Audio");
        block->src_ = getAttribute(elem, "src");
        if (block->src_.empty()) {
            lxb_dom_node_t* c = node->first_child;
            while (c) {
                if (c->type == LXB_DOM_NODE_TYPE_ELEMENT && lxb_dom_node_tag_id(c) == LXB_TAG_SOURCE) {
                    block->src_ = getAttribute(lxb_dom_interface_element(c), "src");
                    if (!block->src_.empty()) break;
                }
                c = c->next;
            }
        }
        blocks.push_back(block);
        return;
    }

    // Separator: <hr>
    if (tagId == LXB_TAG_HR) {
        auto block = std::make_shared<HybridContentBlock>("Separator");
        blocks.push_back(block);
        return;
    }

    // Standard structural containers -> recurse into children
    if (tagId == LXB_TAG_DIV || tagId == LXB_TAG_SECTION || tagId == LXB_TAG_ARTICLE ||
        tagId == LXB_TAG_MAIN || tagId == LXB_TAG_HEADER || tagId == LXB_TAG_FOOTER ||
        tagId == LXB_TAG_ASIDE || tagId == LXB_TAG_NAV || tagId == LXB_TAG_BODY ||
        tagId == LXB_TAG_HTML || tagId == LXB_TAG_CENTER || tagId == LXB_TAG_FORM ||
        tagId == LXB_TAG_FIELDSET) {
        walkDomChildren(node, blocks);
        return;
    }

    // Custom or unrecognized tags (e.g. <custom-poll>, <custom-widget>)
    std::string tagName = getNodeTagName(node);
    if (!tagName.empty() && tagName.find('-') != std::string::npos) {
        auto block = std::make_shared<HybridContentBlock>(tagName);
        block->src_ = getAttribute(elem, "src");
        block->title_ = getAttribute(elem, "title");
        block->alt_ = getAttribute(elem, "id");
        collectInlineChildren(node, block->children_);
        blocks.push_back(block);
        return;
    }

    // Fallback: If it has children, parse as children or fallback to paragraph
    std::vector<std::shared_ptr<HybridInlineNode>> inlines;
    collectInlineChildren(node, inlines);
    if (!inlines.empty()) {
        auto p = std::make_shared<HybridContentBlock>("Paragraph");
        p->children_ = std::move(inlines);
        blocks.push_back(p);
    }
}

// ── estimateHeight ────────────────────────────────────────────────────────────
double HybridFastHtmlParser::estimateHeight(const std::string& html, double lineHeight) {
    if (html.empty()) return 0.0;

    struct TagWeight { const char* tag; double weight; };
    static const std::vector<TagWeight> tags = {
        {"<p",           1.0}, {"<div",         1.0},
        {"<section",     1.0}, {"<article",     1.0},
        {"<main",        1.0}, {"<header",      1.0},
        {"<footer",      1.0}, {"<aside",       1.0},
        {"<nav",         1.0},
        {"<h1",          1.5}, {"<h2",          1.5}, {"<h3",         1.5},
        {"<h4",          1.5}, {"<h5",          1.5}, {"<h6",         1.5},
        {"<li",          1.0}, {"<dt",          1.0}, {"<dd",         1.0},
        {"<tr",          1.0},
        {"<blockquote",  2.0}, {"<pre",         2.0}, {"<details",    2.0},
        {"<summary",     1.0}, {"<figcaption",  1.0}, {"<caption",    1.0},
        {"<figure",      6.0}, {"<svg",         6.0}, {"<canvas",     6.0},
        {"<img",         8.0}, {"<video",      10.0}, {"<iframe",    15.0},
        {"<audio",       2.0},
        {"<hr",          0.5},
        {"<form",        2.0}, {"<fieldset",    2.0},
        {"<input",       1.0}, {"<select",      1.0},
        {"<button",      1.0}, {"<label",       1.0},
        {"<textarea",    3.0},
    };

    double totalUnits = 0.0;
    for (const auto& tw : tags) {
        size_t pos = 0;
        while ((pos = html.find(tw.tag, pos)) != std::string::npos) {
            totalUnits += tw.weight;
            pos += strlen(tw.tag);
        }
    }

    if (totalUnits <= 0.0) {
        return lineHeight;
    }

    return totalUnits * lineHeight * 1.4;
}

// ── parseInternal ─────────────────────────────────────────────────────────────
std::shared_ptr<HybridParsedArticle> HybridFastHtmlParser::parseInternal(const std::string& html) {
    if (html.empty()) {
        return std::make_shared<HybridParsedArticle>();
    }

    lxb_html_parser_t* parser = lxb_html_parser_create();
    if (!parser) {
        return std::make_shared<HybridParsedArticle>();
    }

    lxb_status_t status = lxb_html_parser_init(parser);
    if (status != LXB_STATUS_OK) {
        lxb_html_parser_destroy(parser);
        return std::make_shared<HybridParsedArticle>();
    }

    lxb_html_document_t* document = lxb_html_parse(
        parser,
        reinterpret_cast<const lxb_char_t*>(html.c_str()),
        html.length()
    );

    auto blocks = std::make_shared<std::vector<std::shared_ptr<HybridContentBlock>>>();

    if (document) {
        lxb_dom_node_t* rootNode = nullptr;
        if (document->body) {
            rootNode = lxb_dom_interface_node(document->body);
        } else if (document->dom_document.element) {
            rootNode = lxb_dom_interface_node(document->dom_document.element);
        }

        if (rootNode) {
            walkDomChildren(rootNode, *blocks);
        }

        lxb_html_document_destroy(document);
    }

    lxb_html_parser_destroy(parser);

    return std::make_shared<HybridParsedArticle>(std::move(*blocks));
}

// ── parse (sync) ──────────────────────────────────────────────────────────────
std::variant<std::shared_ptr<HybridParsedArticleSpec>, NullType>
HybridFastHtmlParser::parse(const std::string& html) {
    if (html.empty()) return nullptr;
    auto article = parseInternal(html);
    return article;
}

// ── parseAsync ────────────────────────────────────────────────────────────────
std::shared_ptr<Promise<std::variant<std::shared_ptr<HybridParsedArticleSpec>, NullType>>>
HybridFastHtmlParser::parseAsync(const std::string& html) {
    return Promise<std::variant<std::shared_ptr<HybridParsedArticleSpec>, NullType>>::async([html]() -> std::variant<std::shared_ptr<HybridParsedArticleSpec>, NullType> {
        if (html.empty()) return nullptr;
        return parseInternal(html);
    });
}

// ── parseToJSON ───────────────────────────────────────────────────────────────
std::string HybridFastHtmlParser::parseToJSON(const std::string& html) {
    auto article = parseInternal(html);
    return article ? article->toJSON() : "{\"blocks\":[]}";
}

} // namespace margelo::nitro::fasthtmlparser
