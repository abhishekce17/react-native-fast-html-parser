import React from 'react';
import { StyleSheet, Text, View, ScrollView } from 'react-native';
import { parseHTML } from 'react-native-fast-html-parser';

const sampleHtml = `
  <h1>Fast HTML Parser</h1>
  <p>This is parsed via <b>JSI</b> and <b>Rust</b> with zero copy!</p>
  <pre><code class="typescript">const parsed = parseHTML(html);</code></pre>
`;

export default function App() {
  const article = React.useMemo(() => parseHTML(sampleHtml), []);

  const renderBlocks = () => {
    if (!article) {
      return <Text style={styles.error}>Failed to parse HTML</Text>;
    }

    const blocks = [];
    for (let i = 0; i < article.length; i++) {
      const block = article.getBlock(i);
      if (!block) continue;

      if (block.type === 'Heading') {
        const title = block.getChild(0)?.text || '';
        blocks.push(
          <Text key={i} style={styles.heading}>
            {title}
          </Text>
        );
      } else if (block.type === 'Paragraph') {
        const textParts = [];
        for (let j = 0; j < block.childCount; j++) {
          const inline = block.getChild(j);
          if (!inline) continue;
          if (inline.type === 'Bold') {
            textParts.push(
              <Text key={j} style={styles.bold}>
                {inline.text}
              </Text>
            );
          } else {
            textParts.push(<Text key={j}>{inline.text}</Text>);
          }
        }
        blocks.push(
          <Text key={i} style={styles.paragraph}>
            {textParts}
          </Text>
        );
      } else if (block.type === 'CodeBlock') {
        blocks.push(
          <View key={i} style={styles.codeContainer}>
            <Text style={styles.codeText}>{block.code}</Text>
          </View>
        );
      }
    }
    return blocks;
  };

  return (
    <ScrollView contentContainerStyle={styles.container}>
      <Text style={styles.title}>JSI Demo App</Text>
      <View style={styles.card}>{renderBlocks()}</View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flexGrow: 1,
    backgroundColor: '#f5f7fb',
    padding: 20,
    paddingTop: 60,
  },
  title: {
    fontSize: 28,
    fontWeight: 'bold',
    color: '#1e293b',
    marginBottom: 20,
    textAlign: 'center',
  },
  card: {
    backgroundColor: '#ffffff',
    borderRadius: 12,
    padding: 16,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.05,
    shadowRadius: 8,
    elevation: 2,
  },
  heading: {
    fontSize: 22,
    fontWeight: 'bold',
    color: '#0f172a',
    marginVertical: 10,
  },
  paragraph: {
    fontSize: 16,
    color: '#334155',
    lineHeight: 24,
    marginVertical: 8,
  },
  bold: {
    fontWeight: 'bold',
    color: '#0f172a',
  },
  codeContainer: {
    backgroundColor: '#f1f5f9',
    padding: 12,
    borderRadius: 8,
    marginVertical: 8,
  },
  codeText: {
    fontFamily: 'Courier',
    fontSize: 14,
    color: '#0f172a',
  },
  error: {
    color: '#ef4444',
    textAlign: 'center',
  },
});
