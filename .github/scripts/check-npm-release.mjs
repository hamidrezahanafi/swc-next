#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const [coreBindingsRoot, parserBindingsRoot, corePackageRoot, parserPackageRoot] =
  process.argv.slice(2).map((entry) => path.resolve(entry));

if (!parserPackageRoot) {
  throw new Error(
    "usage: check-npm-release.mjs <core-bindings> <parser-bindings> <core-package> <parser-package>",
  );
}

const suffixes = [
  "darwin-arm64",
  "darwin-x64",
  "linux-arm64-gnu",
  "linux-x64-gnu",
  "win32-x64-msvc",
];

function manifestAt(packageRoot) {
  return JSON.parse(fs.readFileSync(path.join(packageRoot, "package.json"), "utf8"));
}

function findPackageRoots(directory) {
  const roots = [];
  const visit = (entry) => {
    for (const child of fs.readdirSync(entry, { withFileTypes: true })) {
      if (!child.isDirectory()) continue;
      const childPath = path.join(entry, child.name);
      if (fs.existsSync(path.join(childPath, "package.json"))) roots.push(childPath);
      else visit(childPath);
    }
  };
  visit(directory);
  return roots;
}

function dryRunPack(packageRoot) {
  const output = execFileSync("npm", ["pack", "--dry-run", "--json", packageRoot], {
    encoding: "utf8",
  });
  const result = JSON.parse(output)[0];
  if (!result || !Array.isArray(result.files) || result.files.length === 0) {
    throw new Error(`npm pack produced no files for ${packageRoot}`);
  }
  return result.files.map(({ path: filePath }) => filePath);
}

function checkComponent(component, bindingsRoot, packageRoot) {
  const facade = manifestAt(packageRoot);
  const version = facade.version;
  const prefix = `@swc-next/${component}-binding`;
  const expectedBindings = new Map(
    suffixes.map((suffix) => [`${prefix}-${suffix}`, version]),
  );

  if (facade.name !== `@swc-next/${component}`) {
    throw new Error(`unexpected ${component} facade name: ${facade.name}`);
  }
  if (
    !facade.optionalDependencies ||
    Object.keys(facade.optionalDependencies).length !== expectedBindings.size ||
    [...expectedBindings].some(
      ([name, expectedVersion]) => facade.optionalDependencies[name] !== expectedVersion,
    )
  ) {
    throw new Error(`${facade.name} optionalDependencies are incomplete`);
  }

  const bindingRoots = findPackageRoots(bindingsRoot);
  const actualNames = new Set();
  for (const bindingRoot of bindingRoots) {
    const binding = manifestAt(bindingRoot);
    if (!expectedBindings.has(binding.name)) continue;
    actualNames.add(binding.name);
    if (binding.version !== version) {
      throw new Error(`${binding.name} is ${binding.version}, expected ${version}`);
    }
    const packedFiles = dryRunPack(bindingRoot);
    if (packedFiles.filter((filePath) => filePath.endsWith(".node")).length !== 1) {
      throw new Error(`${binding.name} must contain exactly one .node binary`);
    }
  }

  if (actualNames.size !== expectedBindings.size) {
    const missing = [...expectedBindings.keys()].filter((name) => !actualNames.has(name));
    throw new Error(`missing ${component} bindings: ${missing.join(", ")}`);
  }

  const facadeFiles = dryRunPack(packageRoot);
  if (facadeFiles.some((filePath) => filePath.endsWith(".node"))) {
    throw new Error(`${facade.name} facade must not contain native binaries`);
  }
}

checkComponent("core", coreBindingsRoot, corePackageRoot);
checkComponent("parser", parserBindingsRoot, parserPackageRoot);
console.log("Validated core and parser npm release packages.");
