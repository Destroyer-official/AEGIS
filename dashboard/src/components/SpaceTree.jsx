/**
 * SpaceTree.jsx — IPv6 Space Tree Visualization Component
 * 
 * §3.1.2: Divisive hierarchical clustering space tree
 * Integrates with FastAPI /space-tree endpoint for prefix clustering drill-down
 * 
 * Usage:
 * <SpaceTree apiUrl="/space-tree?root_prefix=2001:db8::/32&max_depth=4" />
 */

import React, { useState, useEffect, useMemo } from 'react';
import './SpaceTree.css';

const API_BASE = process.env.REACT_APP_API_BASE || '';

function SpaceTree({ apiUrl = '/space-tree', rootPrefix = '2001:db8::/32', maxDepth = 4, splitThreshold = 5 }) {
  const [treeData, setTreeData] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [selectedNode, setSelectedNode] = useState(null);
  const [expandedNodes, setExpandedNodes] = useState(new Set());

  // Build full API URL with params
  const fullUrl = useMemo(() => {
    const params = new URLSearchParams({
      root_prefix: rootPrefix,
      max_depth: maxDepth.toString(),
      split_threshold: splitThreshold.toString(),
    });
    return `${API_BASE}${apiUrl}?${params.toString()}`;
  }, [apiUrl, rootPrefix, maxDepth, splitThreshold]);

  useEffect(() => {
    let mounted = true;
    setError(null);

    fetch(fullUrl)
      .then(res => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return res.json();
      })
      .then(data => {
        if (mounted) {
          setTreeData(data);
          setLoading(false);
        }
      })
      .catch(err => {
        if (mounted) {
          setError(err.message);
          setLoading(false);
        }
      });

    return () => { mounted = false; };
  }, [fullUrl]);

  const toggleNode = (node) => {
    setExpandedNodes(prev => {
      const next = new Set(prev);
      const key = `${node.prefix}-${node.depth}`;
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  };

  const selectNode = (node) => {
    setSelectedNode(node);
  };

  const isExpanded = (node) => {
    const key = `${node.prefix}-${node.depth}`;
    return expandedNodes.has(key);
  };

  if (loading) return <div className="space-tree-loading">Loading space tree...</div>;
  if (error) return <div className="space-tree-error">Error: {error}</div>;
  if (!treeData) return <div className="space-tree-empty">No data</div>;

  return (
    <div className="space-tree-container">
      <div className="space-tree-header">
        <h3>IPv6 Space Tree — {rootPrefix}</h3>
        <div className="space-tree-stats">
          <span>Nodes: {treeData.total_nodes}</span>
          <span>Leaves: {treeData.leaf_count}</span>
          <span>Max Depth: {treeData.max_depth}</span>
        </div>
      </div>

      <div className="space-tree-main">
        <div className="space-tree-canvas">
          <TreeNode
            node={treeData.root}
            depth={0}
            isExpanded={isExpanded}
            onToggle={toggleNode}
            onSelect={selectNode}
            selectedNode={selectedNode}
          />
        </div>

        {selectedNode && (
          <div className="space-tree-details">
            <h4>Prefix Details</h4>
            <table className="space-tree-detail-table">
              <tbody>
                <tr><td>Prefix</td><td><code>{selectedNode.prefix}</code></td></tr>
                <tr><td>Prefix Length</td><td>/ {selectedNode.prefix_len}</td></tr>
                <tr><td>Depth</td><td>{selectedNode.depth}</td></tr>
                <tr><td>Seed Count</td><td>{selectedNode.seed_count}</td></tr>
                <tr><td>Hit Rate</td><td>{(selectedNode.hit_rate * 100).toFixed(2)}%</td></tr>
                <tr><td>Expected Reward</td><td>{selectedNode.expected_reward.toFixed(2)}</td></tr>
                <tr><td>Status</td><td>
                  <span className={selectedNode.saturated ? 'status-saturated' : 'status-active'}>
                    {selectedNode.saturated ? 'Saturated' : 'Active'}
                  </span>
                </td></tr>
                <tr><td>Children</td><td>{selectedNode.children?.length || 0}</td></tr>
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}

function TreeNode({ node, depth, isExpanded, onToggle, onSelect, selectedNode }) {
  const hasChildren = node.children && node.children.length > 0;
  const expanded = isExpanded(node);
  const selected = selectedNode && selectedNode.prefix === node.prefix;

  return (
    <div className={`tree-node ${expanded ? 'expanded' : ''} ${selected ? 'selected' : ''} depth-${depth}`}>
      <div 
        className="tree-node-header"
        onClick={() => { onSelect(node); if (hasChildren) onToggle(node); }}
        style={{ paddingLeft: `${depth * 20 + 10}px` }}
      >
        <span className="tree-node-toggle">
          {hasChildren ? (expanded ? '▼' : '▶') : '•'}
        </span>
        <span className="tree-node-prefix">{node.prefix}</span>
        <span className="tree-node-meta">
          seeds: {node.seed_count} | hit: {(node.hit_rate * 100).toFixed(1)}% | reward: {node.expected_reward.toFixed(1)}
        </span>
        {node.saturated && <span className="tree-node-badge saturated">SATURATED</span>}
      </div>

      {expanded && hasChildren && (
        <div className="tree-node-children">
          {node.children.map((child, idx) => (
            <TreeNode
              key={`${child.prefix}-${idx}`}
              node={child}
              depth={depth + 1}
              isExpanded={isExpanded}
              onToggle={onToggle}
              onSelect={onSelect}
              selectedNode={selectedNode}
            />
          ))}
        </div>
      )}
    </div>
  );
}

export default SpaceTree;