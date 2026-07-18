// Tokens + base reset lead the CSS bundle so component styles resolve against them.
import "./styles/tokens.css";
import "./styles/global.css";

// Core UI
export { Button } from "./components/Button/Button";
export type { ButtonProps } from "./components/Button/Button";
export { Tag } from "./components/Tag/Tag";
export type { TagProps } from "./components/Tag/Tag";
export { StatusBadge } from "./components/StatusBadge/StatusBadge";
export type { StatusBadgeProps } from "./components/StatusBadge/StatusBadge";
export { NavBar } from "./components/NavBar/NavBar";
export type { NavBarProps, NavBarLink } from "./components/NavBar/NavBar";
export { SiteFooter } from "./components/SiteFooter/SiteFooter";
export type { SiteFooterProps } from "./components/SiteFooter/SiteFooter";

// Site patterns
export { SectionHeading } from "./components/SectionHeading/SectionHeading";
export type { SectionHeadingProps } from "./components/SectionHeading/SectionHeading";
export { ImpactStat } from "./components/ImpactStat/ImpactStat";
export type { ImpactStatProps } from "./components/ImpactStat/ImpactStat";
export { ProjectRow } from "./components/ProjectRow/ProjectRow";
export type { ProjectRowProps } from "./components/ProjectRow/ProjectRow";
export { DiagnosticsPanel } from "./components/DiagnosticsPanel/DiagnosticsPanel";
export type {
  DiagnosticsPanelProps,
  DiagnosticItem,
} from "./components/DiagnosticsPanel/DiagnosticsPanel";
export { Metric } from "./components/Metric/Metric";
export type { MetricProps } from "./components/Metric/Metric";

// Article / reading
export { ArticleHeader } from "./components/ArticleHeader/ArticleHeader";
export type { ArticleHeaderProps } from "./components/ArticleHeader/ArticleHeader";
export { TableOfContents } from "./components/TableOfContents/TableOfContents";
export type { TableOfContentsProps, TocEntry } from "./components/TableOfContents/TableOfContents";
export { CodeBlock } from "./components/CodeBlock/CodeBlock";
export type { CodeBlockProps } from "./components/CodeBlock/CodeBlock";
export { Callout } from "./components/Callout/Callout";
export type { CalloutProps } from "./components/Callout/Callout";
export { Prose } from "./components/Prose/Prose";
export type { ProseProps } from "./components/Prose/Prose";
