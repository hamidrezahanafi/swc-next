#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const repositoryRoot = path.resolve(scriptDirectory, "../../../..");
const rootNpmManifestPath = "package.json";
const npmManifestPaths = ["napi/core/package.json", "napi/parser/package.json"];
const expectedTargets = [
  "aarch64-apple-darwin",
  "aarch64-unknown-linux-gnu",
  "x86_64-apple-darwin",
  "x86_64-pc-windows-msvc",
  "x86_64-unknown-linux-gnu",
];

function fail(message) {
  console.error(`release-version: ${message}`);
  process.exit(1);
}

function read(relativePath) {
  return fs.readFileSync(path.join(repositoryRoot, relativePath), "utf8");
}

function write(relativePath, content) {
  fs.writeFileSync(path.join(repositoryRoot, relativePath), content);
}

function cargoManifestPaths() {
  return ["crates", "napi"].flatMap((directory) =>
    fs
      .readdirSync(path.join(repositoryRoot, directory), { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .map((entry) => `${directory}/${entry.name}/Cargo.toml`)
      .filter((relativePath) => fs.existsSync(path.join(repositoryRoot, relativePath)))
      .filter((relativePath) => {
        const manifest = read(relativePath);
        return /^name\s*=\s*"swc_next_[^"]+"/m.test(manifest) &&
          !/^publish\s*=\s*false/m.test(manifest);
      }),
  );
}

function cargoPackage(manifestPath) {
  const manifest = read(manifestPath);
  const name = manifest.match(/^name\s*=\s*"([^"]+)"/m)?.[1];
  const version = manifest.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
  if (!name || !version) fail(`missing package name or version in ${manifestPath}`);
  return { manifestPath, name, version };
}

function parseVersion(version) {
  const match = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.exec(version);
  if (!match) fail(`unsupported semantic version: ${version}`);
  return match.slice(1).map(Number);
}

function bumpVersion(version, bump) {
  const [major, minor, patch] = parseVersion(version);
  if (bump === "major") return `${major + 1}.0.0`;
  if (bump === "minor") return `${major}.${minor + 1}.0`;
  if (bump === "patch") return `${major}.${minor}.${patch + 1}`;
  fail(`bump must be patch, minor, or major; received ${bump}`);
}

function currentVersion() {
  const packages = cargoManifestPaths().map(cargoPackage);
  const versions = new Set(packages.map(({ version }) => version));
  for (const manifestPath of npmManifestPaths) {
    versions.add(JSON.parse(read(manifestPath)).version);
  }
  versions.add(JSON.parse(read(rootNpmManifestPath)).version);
  if (versions.size !== 1) {
    fail(`release versions are not synchronized: ${[...versions].join(", ")}`);
  }
  return [...versions][0];
}

function replaceCargoVersion(manifestPath, version) {
  const manifest = read(manifestPath);
  const updated = manifest.replace(
    /^(version\s*=\s*")[^"]+(".*)$/m,
    `$1${version}$2`,
  );
  if (updated === manifest) fail(`failed to update ${manifestPath}`);
  write(manifestPath, updated);
}

function updateWorkspaceDependencies(version) {
  const manifest = read("Cargo.toml");
  let replacements = 0;
  const updated = manifest.replace(
    /^(swc_next_[\w]+\s*=\s*\{[^\n]*version\s*=\s*")[^"]+("[^\n]*\})$/gm,
    (_, prefix, suffix) => {
      replacements += 1;
      return `${prefix}${version}${suffix}`;
    },
  );
  if (replacements === 0) fail("found no swc_next_* workspace dependency versions");
  write("Cargo.toml", updated);
}

function updateNpmManifest(manifestPath, version) {
  const manifest = JSON.parse(read(manifestPath));
  manifest.version = version;
  write(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
}

function updateVersion(version) {
  parseVersion(version);
  for (const manifestPath of cargoManifestPaths()) {
    replaceCargoVersion(manifestPath, version);
  }
  updateWorkspaceDependencies(version);
  for (const manifestPath of npmManifestPaths) {
    updateNpmManifest(manifestPath, version);
  }
  updateNpmManifest(rootNpmManifestPath, version);
}

function checkCargoLock(packages, version) {
  const lockfile = read("Cargo.lock");
  for (const { name } of packages) {
    const escapedName = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const entry = new RegExp(
      `\\[\\[package\\]\\]\\nname = "${escapedName}"\\nversion = "([^"]+)"`,
    ).exec(lockfile);
    if (!entry) fail(`Cargo.lock has no entry for ${name}`);
    if (entry[1] !== version) {
      fail(`Cargo.lock has ${name}@${entry[1]}, expected ${version}`);
    }
  }
}

function checkWorkspaceDependencies(version) {
  const manifest = read("Cargo.toml");
  for (const match of manifest.matchAll(
    /^(swc_next_[\w]+)\s*=\s*\{[^\n]*version\s*=\s*"([^"]+)"[^\n]*\}$/gm,
  )) {
    if (match[2] !== version) {
      fail(`workspace dependency ${match[1]} is ${match[2]}, expected ${version}`);
    }
  }
}

function checkNpmManifest(manifestPath, version) {
  const manifest = JSON.parse(read(manifestPath));
  if (manifest.version !== version) {
    fail(`${manifest.name} is ${manifest.version}, expected ${version}`);
  }
  if ("optionalDependencies" in manifest) {
    fail(`${manifest.name} must generate optionalDependencies during release`);
  }
  if (JSON.stringify(manifest.napi.targets) !== JSON.stringify(expectedTargets)) {
    fail(`${manifest.name} does not declare the supported five-target matrix`);
  }
  const component = manifest.name === "@swc-next/core" ? "core" : "parser";
  if (manifest.napi.packageName !== `@swc-next/${component}-binding`) {
    fail(`${manifest.name} has an unexpected N-API package name`);
  }
  const bindingPath = `napi/${component}/src-js/binding.js`;
  const binding = read(bindingPath);
  if (!binding.includes(`@swc-next/${component}-binding-`)) {
    fail(`${bindingPath} does not load the ${component} binding family`);
  }
  if (!binding.includes(`expected ${version} but got`)) {
    fail(`${bindingPath} was not regenerated for ${version}`);
  }
}

function checkVersion(expectedVersion) {
  const packages = cargoManifestPaths().map(cargoPackage);
  const version = currentVersion();
  if (expectedVersion && expectedVersion !== version) {
    fail(`repository version is ${version}, expected ${expectedVersion}`);
  }
  checkWorkspaceDependencies(version);
  checkCargoLock(packages, version);
  const rootManifest = JSON.parse(read(rootNpmManifestPath));
  if (!rootManifest.private || rootManifest.version !== version) {
    fail(`private workspace root must use release version ${version}`);
  }
  for (const manifestPath of npmManifestPaths) {
    checkNpmManifest(manifestPath, version);
  }
  console.log(version);
}

const [command, ...arguments_] = process.argv.slice(2);
if (command === "current") {
  console.log(currentVersion());
} else if (command === "bump") {
  console.log(bumpVersion(currentVersion(), arguments_[0]));
} else if (command === "write") {
  updateVersion(arguments_[0]);
  console.log(arguments_[0]);
} else if (command === "check") {
  checkVersion(arguments_[0]);
} else if (command === "compare") {
  const [left, right] = arguments_.map(parseVersion);
  const comparison = left.findIndex((part, index) => part !== right[index]);
  if (comparison === -1 || left[comparison] >= right[comparison]) {
    fail(`${arguments_[1]} must be greater than ${arguments_[0]}`);
  }
} else {
  fail("usage: release-version.mjs current|bump <kind>|write <version>|check [version]|compare <old> <new>");
}
