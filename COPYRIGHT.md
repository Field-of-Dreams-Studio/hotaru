# Hotaru Framework Patent Application Guide
# Hotaru 框架专利申请指南

> **Document Version**: 1.0  
> **Last Updated**: 2025-01-26  
> **Status**: CONFIDENTIAL - INTERNAL USE ONLY

---

## 📋 Executive Summary / 执行摘要

Hotaru framework contains innovative technologies eligible for patent protection in multiple jurisdictions. This document provides operational guidelines for patent applications.

Hotaru 框架包含多项可申请专利保护的创新技术。本文档提供专利申请的操作指南。

---

## 🎯 Core Technical Innovations / 核心技术创新

### 1. Protocol-Agnostic Abstraction Layer / 协议无关抽象层
- **Location / 代码位置**: `hotaru_core/src/connection/*` `hotaru_core/src/app/*`
- **Key Innovation / 关键创新**: Unified trait system for any protocol implementation
- **Patent Focus / 专利重点**: Protocol trait, Transport, Stream, Message abstractions

### 2. Compile-Time Code Generation / 编译时代码生成
- **Location / 代码位置**: `hotaru_meta/src/lib.rs`
- **Key Innovation / 关键创新**: Macro-driven protocol-specific handler generation
- **Patent Focus / 专利重点**: `endpoint!` and `middleware!` macros

### 3. Automatic Protocol Detection & Routing / 自动协议检测与路由
- **Location / 代码位置**: `hotaru_core/src/app/protocol.rs`
- **Key Innovation / 关键创新**: Runtime protocol identification without performance penalty
- **Patent Focus / 专利重点**: ProtocolRegistry, detection algorithm

---

## 🌍 Patent Application Strategy / 专利申请策略

### ⚡ Updated Timeline: Patent Before Open Source / 更新时间线：先专利后开源

**Critical Dates / 关键日期**:
- **Patent Filing Deadline / 专利申请截止**: September 1, 2025 / 2025年9月1日
- **Open Source Release / 开源发布**: September 3, 2025 / 2025年9月3日  
- **PCT Filing / PCT申请**: Within 10 months (by July 2026) / 10个月内（2026年7月前）

### Phase 1: Priority Filing (August-September 2025) / 第一阶段：优先权申请（2025年8-9月）

#### 📅 Execution Timeline / 执行时间表

| Date / 日期 | Action / 行动 | Priority / 优先级 |
|------------|---------------|-------------------|
| **Feb-May 2025** | Technical documentation preparation / 技术文档准备 | High |
| **Jun-Jul 2025** | Patent drafting / 专利撰写 | High |
| **Aug 1-15** | Final review and translation / 最终审核和翻译 | Critical |
| **Aug 25** | Submit China Patent / 提交中国专利 | Critical |
| **Aug 26** | Submit US Provisional / 提交美国临时专利 | Critical |
| **Aug 27-31** | Buffer period / 缓冲期 | High |
| **Sep 1** | **Final Deadline** / **最终截止** | Critical |
| **Sep 2** | Verify filing receipts / 确认受理回执 | High |
| **Sep 3** | **Open Source Release** / **开源发布** | - |

#### 🇨🇳 China Patent Application / 中国专利申请
**Timeline / 时间**: By August 25, 2025 / 2025年8月25日前  
**Patent Title / 专利名称**:
```
一种基于编译时代码生成的多协议网络服务处理系统及方法
```

**Application Type / 申请类型**: 发明专利 (Invention Patent)

**Key Claims Structure / 权利要求结构**:
```
1. 一种多协议网络服务处理系统，其特征在于，包括：
   - 协议抽象层模块，用于提供统一的协议接口；
   - 编译时代码生成模块，用于生成协议特定的处理程序；
   - 协议检测模块，用于识别传入连接的协议类型；
   - 路由模块，用于将连接分配至相应的协议处理器。

2. 根据权利要求1所述的系统，其特征在于，所述协议抽象层模块包括：
   - 传输层抽象，用于管理连接级别状态；
   - 流抽象，用于多路复用通信；
   - 消息抽象，用于协议特定的数据编解码；
   - 请求上下文抽象，用于请求响应类型绑定。

3. 一种多协议网络服务处理方法，其特征在于，包括以下步骤：
   - 接收网络连接；
   - 读取初始字节流进行协议检测；
   - 根据检测结果路由至对应协议处理器；
   - 通过编译时生成的代码执行业务逻辑；
   - 按照协议要求格式化响应。
```

**Technical Effect Emphasis / 技术效果强调**:
- 减少内存占用 30-40%（通过编译时优化）
- 提高请求处理速度 25-35%
- 降低 CPU 使用率 20-30%
- 支持并发连接数提升 2-3 倍

**Required Documents / 所需文件**:
- [ ] 技术交底书（详细技术方案）
- [ ] 说明书（含具体实施例）
- [ ] 权利要求书（10-20 项权利要求）
- [ ] 说明书摘要（< 300 字）
- [ ] 说明书附图（5-8 幅）
- [ ] 专利代理委托书

**Estimated Cost / 预估费用**: 
- 官费：¥3,450（申请费 + 实审费）
- 代理费：¥8,000-15,000
- 加急费（如需）：¥5,000-8,000

#### 🇺🇸 US Provisional Patent Application / 美国临时专利申请
**Timeline / 时间**: By August 26, 2025 / 2025年8月26日前   
**Patent Title / 专利名称**:
```
System and Method for Protocol-Agnostic Web Service Framework with Compile-Time Code Generation
```

**Key Components / 主要内容**:
1. Detailed technical specification
2. Implementation examples (HTTP/1.1, HTTP/2, WebSocket)
3. Performance benchmarks and comparisons
4. Drawings (can be informal for provisional)

**Required Documents / 所需文件**:
- [ ] Technical specification document
- [ ] Cover sheet with inventor information
- [ ] Filing fee: $320 USD (small entity: $160)
- [ ] No formal claims required (but recommended)
- [ ] Express filing if needed: +$400-800

**Advantages / 优势**:
- Establishes early priority date
- 12-month grace period before non-provisional
- Lower initial cost
- Can be filed quickly

**Important Notes / 注意事项**:
- Does NOT need formal patent claims
- Can use existing technical documentation
- Must be converted within 12 months
- US has 12-month grace period but file before open source to be safe

### Phase 2: Open Source Release (September 2025) / 第二阶段：开源发布（2025年9月）

#### 🚀 Pre-Release Checklist / 发布前检查清单
- [ ] Confirm China patent application number received
- [ ] Confirm US provisional filing receipt received  
- [ ] Review code for any trade secrets to keep private
- [ ] Prepare open source license (MIT recommended)
- [ ] Create release notes mentioning "Patent Pending"

#### 📊 Release Strategy / 发布策略
1. **Staged Release / 分阶段发布**:
   - Sep 3: Core functionality / 核心功能
   - Sep 15: Complete framework / 完整框架
   - Sep 30: Advanced examples / 高级示例

2. **Community Engagement / 社区运营**:
   - Collect user feedback for PCT optimization
   - Document performance improvements
   - Track adoption metrics

### Phase 3: PCT International Filing (By July 2026) / 第三阶段：PCT国际申请（2026年7月前）

#### 🌐 PCT Application / PCT国际申请
**Timeline / 时间**: By July 2026 (within 10 months) / 2026年7月前（10个月内）
**Hard Deadline / 硬性截止**: September 1, 2026 (12 months) / 2026年9月1日（12个月）  
**Patent Title / 专利名称**:
```
Protocol-Agnostic Web Service Framework with Compile-Time Optimization
```

**Priority Claims / 优先权要求**:
- China Patent Application (Filed Aug 25, 2025)
- US Provisional Patent (Filed Aug 26, 2025)

**PCT Application Strategy / PCT申请策略**:
1. **International Searching Authority / 国际检索单位**: 
   - Recommend: CNIPA (China) or USPTO (USA)
   - Consider search report quality and cost

2. **Languages / 语言**:
   - File in English (for broader examination)
   - Chinese translation for CNIPA ISA

3. **Claims Strategy / 权利要求策略**:
   - Incorporate feedback from open source community
   - 15-20 claims recommended
   - Include improvements discovered post-release

**Required Documents / 所需文件**:
- [ ] PCT Request Form (PCT/RO/101)
- [ ] Complete specification with formal claims
- [ ] Priority documents (CN and US)
- [ ] Power of Attorney for PCT agent
- [ ] Filing fee: ~$3,000-4,000 USD

**Timeline Benefits / 时间优势**:
- Delays national phase to 30/31 months from priority
- Provides 18+ months to evaluate markets
- Allows refinement based on search report

### Phase 4: National Phase Entry (2027-2028) / 第四阶段：国家阶段进入（2027-2028）

**National Phase Deadline / 国家阶段截止**: 30/31 months from priority date
- China priority date: August 25, 2025
- Deadline: February/March 2028

#### Target Countries Priority / 目标国家优先级

**Tier 1 (Must Enter) / 第一梯队（必须进入）**:
- 🇺🇸 **United States** (convert provisional to non-provisional)
- 🇪🇺 **European Patent Office** (covers EU members)
- 🇯🇵 **Japan** (major tech market)

**Tier 2 (Recommended) / 第二梯队（建议进入）**:
- 🇰🇷 **South Korea** (tech hub)
- 🇬🇧 **United Kingdom** (post-Brexit separate filing)
- 🇨🇦 **Canada** (North American coverage)
- 🇦🇺 **Australia** (APAC presence)

**Tier 3 (Evaluate Based on Adoption) / 第三梯队（根据采用情况评估）**:
- 🇮🇳 **India** (growing tech market)
- 🇧🇷 **Brazil** (Latin America)
- 🇮🇱 **Israel** (tech innovation hub)
- 🇸🇬 **Singapore** (ASEAN gateway)

**National Phase Deadlines / 国家阶段截止日期**:
- 30 months: China, Europe, Japan, Korea
- 31 months: United States, Canada
- Varies: Other jurisdictions

---

## 📝 Immediate Action Items / 立即行动事项

### Preparation Phase (February-July 2025) / 准备阶段（2-7月）

**February-May 2025 / 2025年2-5月**:
- [ ] Complete technical documentation
- [ ] Collect performance benchmarks
- [ ] Document implementation examples
- [ ] Prepare architecture diagrams

**June-July 2025 / 2025年6-7月**:
- [ ] Engage patent attorneys (CN and US)
- [ ] Draft Chinese patent specification
- [ ] Prepare US provisional documentation
- [ ] Create formal drawings (5-8 figures)
- [ ] Draft claims (10-20 items)

### Critical Execution Phase (August 2025) / 关键执行阶段（8月）

**August 1-15 / 8月1-15日**:
- [ ] Final review of all documents
- [ ] Complete translations
- [ ] Attorney final check
- [ ] Prepare filing documents

**August 25-26 / 8月25-26日**:
- [ ] Submit China patent (Aug 25)
- [ ] Submit US provisional (Aug 26)
- [ ] Obtain filing receipts
- [ ] Verify application numbers

### Post-Filing Actions (September 2025-July 2026) / 申请后行动（9月-次年7月）

**September 2025 / 2025年9月**:
- [ ] Open source release (Sep 3)
- [ ] Community engagement
- [ ] Collect user feedback

**October 2025-June 2026 / 2025年10月-2026年6月**:
- [ ] Monitor patent status
- [ ] Document improvements from community
- [ ] Prepare PCT application

**July 2026 / 2026年7月**:
- [ ] File PCT application
- [ ] Claim dual priorities (CN + US)
- [ ] Request international search

---

## 📊 Budget Planning / 预算规划

### Phase 1 Costs (August 2025) / 第一阶段费用（8月）

| Item / 项目 | Cost / 费用 | Notes / 备注 |
|------------|-------------|--------------|
| CN Patent Filing / 中国专利申请 | ¥15,000-20,000 | 含官费及代理费 |
| CN Express Service / 中国加急服务 | ¥5,000-8,000 | 如需加快 |
| US Provisional / 美国临时专利 | $2,000-3,000 | 含官费及律师费 |
| US Express Filing / 美国加急 | $400-800 | 如需加快 |
| Technical Documentation / 技术文档 | ¥5,000-8,000 | 翻译及制图 |
| Attorney Rush Fees / 律师加急费 | +30-50% | 8月密集工作 |
| **Total Phase 1 / 第一阶段合计** | **¥80,000-100,000** | 含应急费用 |

### Phase 2 Costs (July 2026) / 第二阶段费用（7月 2026）

| Item / 项目 | Cost / 费用 | Notes / 备注 |
|------------|-------------|--------------|
| PCT Filing / PCT申请 | $3,000-4,000 | 国际申请费 |
| ISA Search / 国际检索 | $1,500-2,000 | 检索费 |
| Attorney Fees / 律师费 | $5,000-8,000 | 专业代理 |
| **Total Phase 2 / 第二阶段合计** | **$9,500-14,000** | ~¥65,000-100,000 |

### Phase 3 Costs (2027-2028) / 第三阶段费用

| Country / 国家 | Estimated Cost / 预估费用 | Notes / 备注 |
|---------------|------------------------|--------------|
| US Non-Provisional / 美国正式 | $8,000-12,000 | 含审查费用 |
| EPO / 欧洲 | €7,000-10,000 | 含翻译验证 |
| Japan / 日本 | ¥600,000-800,000 | 含日文翻译 |
| Others / 其他 | $5,000-8,000 each | 每个国家 |

---

## ⚠️ Critical Deadlines / 关键截止日期

### 2025 Timeline / 2025年时间线
```
February-May 2025 (准备期)
├── 完成技术文档准备
├── 收集性能数据
└── 准备实施例

June-July 2025 (正式准备)
├── 签约专利代理
├── 专利撰写
└── 制作附图

August 25-26, 2025 (🔴 关键日期)
├── 提交中国专利 (8月25日)
├── 提交美国临时专利 (8月26日)
└── 获取申请号

September 3, 2025
├── GitHub开源发布
└── 社区运营开始

July 2026 (建议)
├── PCT申请提交
└── 要求双重优先权

September 1, 2026 (🔴 硬性截止)
├── PCT申请12个月最后期限
└── 不可延期
```

### 2026-2028 Timeline / 2026-2028年时间线
```
December 2026
├── 收到PCT国际检索报告
└── 评估权利要求修改

February 2027 (优先权日起18个月)
├── PCT国际公布
└── 技术公开可查

October 2027
├── 准备国家阶段文件
└── 确定目标国家

February-March 2028 (🔴 优先权日起30/31个月)
├── 国家阶段进入截止
├── 必须完成所有国家申请
└── 缴纳各国申请费用
```

---

## 🚨 Risk Mitigation & Emergency Plan / 风险缓解与应急预案

### Critical Risk Points / 关键风险点

1. **Timing Risk / 时间风险**
   - **Risk**: Patent not filed before Sep 1 deadline
   - **Mitigation**: Start preparation in February, complete by Aug 15
   - **Emergency**: If delayed, **postpone open source release**

2. **Documentation Risk / 文档风险**
   - **Risk**: Incomplete technical documentation
   - **Mitigation**: Use existing code + performance data
   - **Emergency**: US provisional accepts informal documentation

3. **Budget Risk / 预算风险**
   - **Risk**: Insufficient funds for all filings
   - **Mitigation**: Prioritize China patent (lower cost, home market)
   - **Emergency**: File US provisional only ($320), defer China

### Emergency Protocols / 应急协议

#### Scenario 1: Cannot file both patents by Sep 1
**Action Plan**:
1. Prioritize one country (recommend US for grace period)
2. Postpone open source to Sep 10-15
3. Complete second filing within grace period

#### Scenario 2: Technical documentation incomplete
**Action Plan**:
1. File US provisional with available materials
2. China patent requires formal docs - get 1-week extension
3. Use provisional priority for later formal filing

#### Scenario 3: Open source leaked before patents
**Action Plan**:
1. Immediately file US provisional (12-month grace period)
2. File China patent within 6 months (if possible)
3. Document leak date and source for legal records

### Code Repository Protection / 代码库保护

```bash
# Create patent branch before changes
git checkout -b patent-protection-backup
git push origin patent-protection-backup

# Set repository to private
# GitHub Settings > Visibility > Private

# Schedule automatic public release
# Use GitHub Actions for Sep 3 release
```

### Pre-Filing Checklist (August 24, 2025) / 申请前检查清单

- [ ] All core innovation code documented
- [ ] Performance benchmarks collected
- [ ] 5+ implementation examples ready
- [ ] Drawings completed and reviewed
- [ ] Claims reviewed by attorney
- [ ] Filing fees prepared
- [ ] Power of Attorney signed
- [ ] Emergency contact list ready

---

## 📚 Document Templates / 文档模板

### Technical Disclosure Template (Chinese) / 技术交底书模板（中文）

```markdown
# 技术交底书

## 1. 技术领域
本发明涉及网络服务框架技术领域，具体涉及一种多协议网络服务处理系统及方法。

## 2. 背景技术
[描述现有技术问题]

## 3. 发明内容
[描述技术方案]

## 4. 具体实施方式
[提供3-5个实施例]

## 5. 技术效果
[量化的性能提升数据]
```

### US Provisional Template / 美国临时专利模板

```markdown
# PROVISIONAL PATENT APPLICATION

## TITLE
System and Method for Protocol-Agnostic Web Service Framework

## INVENTORS
[List all inventors]

## BACKGROUND
[Technical field and prior art]

## SUMMARY
[Brief description of innovation]

## DETAILED DESCRIPTION
[Complete technical specification]

## EXAMPLES
[Implementation examples]
```

---

## 👥 Recommended Service Providers / 推荐服务商

### Chinese Patent Agents / 中国专利代理

**北京 Beijing**:
- 中国国际贸易促进委员会专利商标事务所
- 北京集佳知识产权代理有限公司

**上海 Shanghai**:
- 上海专利商标事务所有限公司

### US Patent Attorneys / 美国专利律师

**Software Patent Specialists**:
- Fish & Richardson P.C.
- Fenwick & West LLP
- Wilson Sonsini Goodrich & Rosati

### PCT Agents / PCT代理

**International Firms**:
- WIPO recommended agents
- Local agents with PCT experience

---

## 🔒 Confidentiality Reminder / 保密提醒

**CRITICAL WARNING / 重要警告**:
- Any public disclosure (GitHub releases, presentations, papers) may impact patent rights
- Some countries have absolute novelty requirements (no grace period)
- Keep all technical details confidential until patents are filed
- Use NDAs when discussing with third parties

---

## 📌 Quick Reference Summary / 快速参考摘要

### Key Dates / 关键日期
- **Aug 25, 2025**: China patent filing / 中国专利申请
- **Aug 26, 2025**: US provisional filing / 美国临时专利申请
- **Sep 1, 2025**: Final patent deadline / 专利最终截止
- **Sep 3, 2025**: Open source release / 开源发布
- **Jul 2026**: PCT filing (recommended) / PCT申请（建议）
- **Sep 1, 2026**: PCT deadline (hard limit) / PCT截止（硬性）
- **Feb-Mar 2028**: National phase deadline / 国家阶段截止

### Priority Action Items / 优先行动
1. **NOW**: Begin technical documentation / 开始技术文档
2. **Jun 2025**: Engage patent attorneys / 聘请专利代理
3. **Aug 2025**: Complete all filings / 完成所有申请
4. **Sep 2025**: Execute open source strategy / 执行开源策略

### Budget Summary / 预算摘要
- **Phase 1** (Patents): ¥80,000-100,000
- **Phase 2** (PCT): $9,500-14,000
- **Phase 3** (National): $5,000-10,000 per country
- **Total Estimated**: ~$50,000-80,000 USD

### Critical Success Factors / 关键成功因素
✅ Patents filed before open source  
✅ Dual priority (CN + US) established  
✅ PCT filed within 10 months  
✅ Community feedback incorporated  
✅ National phase strategy optimized  

---

**Document Control / 文档控制**:
- Created: 2025-01-26
- Last Modified: 2025-01-26
- Next Review: Monthly until PCT filing
- Classification: CONFIDENTIAL - INTERNAL USE ONLY
- **Action Required**: Begin preparation immediately

---

END OF DOCUMENT / 文档结束